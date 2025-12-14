use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::LazyLock;

use ilhook::{x64::*, *};
use pelite::pe64::Pe;
use shared::{Program, ext::*};

use crate::dlio::*;
use crate::rva;
use crate::sprj::*;

/// A magic header string that we write into save data that's modified using
/// [on_save] so we can tell whether it was modified by our custom code.
const HEADER: &str = "fromsoftware-rs";

static EQUIP_GAME_DATA_DESERIALIZE_VA: LazyLock<u64> = LazyLock::new(|| {
    Program::current()
        .rva_to_va(rva::get().equip_game_data_deserialize)
        .expect("Call target for EQUIP_GAME_DATA_DESERIALIZE_VA was not in exe")
});

/// An enum of different circumstances in which a save file can be loaded.
pub enum OnLoadType<'a> {
    /// The fake save file for the main menu is loading. This happens when the
    /// game starts (after the first button press), and again each time the
    /// player quits their game.
    ///
    /// The [on_save] callback is not run for the main menu save, so this
    /// never has modded data associated with it.
    MainMenu,

    /// A non-menu save file with data written by [on_save] is loading.
    SavedData(&'a [u8]),

    /// A non-menu save file without data written by [on_save] is loading.
    NoSavedData,
}

/// Registers [callback] to run each time DS3 loads a save that's been modified
/// by [on_save].
///
/// This returns an opaque struct that will unregister the hook when dropped.
///
/// ## Callback
///
/// The callback takes a binary slice that contains the data that was returned
/// by the callback to [on_save].
///
/// ## Safety
///
/// This is subject to all the standard [ilhook safety concerns].
///
/// [ilhook safety concerns]: https://docs.rs/ilhook/latest/ilhook/x64/struct.Hooker.html#method.hook
pub unsafe fn on_load<'a, T: Fn(OnLoadType<'_>) + Send + Sync + 'a>(
    callback: T,
) -> Result<ClosureHookPoint<'a>, HookError> {
    let callback = move |reg: *mut Registers, original| {
        let original: extern "win64" fn(&mut EquipGameData, &mut DLMemoryInputStream) -> usize =
            unsafe { std::mem::transmute(original) };
        // Safety: We trust that DS3 gives us valid pointers.
        let this = unsafe { &mut *((*reg).rcx as *mut EquipGameData) };
        let stream = unsafe { &mut *((*reg).rdx as *mut DLMemoryInputStream) };

        let mut header = [0; HEADER.len()];
        let before_header = stream.stream_position().unwrap();
        let has_saved_data =
            stream.read(&mut header).unwrap() == HEADER.len() && header == HEADER.as_bytes();
        if has_saved_data {
            let data = stream.read_delimited().unwrap();
            callback(OnLoadType::SavedData(data.as_ref()));
        } else {
            stream.seek(SeekFrom::Start(before_header)).unwrap();
        }

        if original(this, stream) == 0 {
            return 0;
        }

        if !has_saved_data {
            callback(if this.is_main_menu() {
                OnLoadType::MainMenu
            } else {
                OnLoadType::NoSavedData
            });
        }

        1
    };

    unsafe {
        hook_closure_retn(
            *EQUIP_GAME_DATA_DESERIALIZE_VA as usize,
            callback,
            CallbackOption::None,
            HookFlags::empty(),
        )
    }
}

static EQUIP_GAME_DATA_SERIALIZE_VA: LazyLock<u64> = LazyLock::new(|| {
    Program::current()
        .rva_to_va(rva::get().equip_game_data_serialize)
        .expect("Call target for EQUIP_GAME_DATA_SERIALIZE_VA was not in exe")
});

/// Registers [callback] to run each time DS3 writes a save. This only counts
/// "real" saves—there's a fake save file that's used by the main screen to
/// store configuration, and that doesn't trigger this callback.
///
/// This returns an opaque struct that will unregister the hook when dropped.
///
/// ## Callback
///
/// The callback returns a byte vector which provides arbitrary data to write to
/// the save file, with the caveat that the entire save file can't exceed 2GB.
/// It may also return None, in which case the vanilla save data will be
/// unchanged and [on_load] won't be run when that data is loaded.
///
/// ## Safety
///
/// This is subject to all the standard [ilhook safety concerns].
///
/// [ilhook safety concerns]: https://docs.rs/ilhook/latest/ilhook/x64/struct.Hooker.html#method.hook
pub unsafe fn on_save<'a, T: (Fn() -> Option<Vec<u8>>) + Send + Sync + 'a>(
    callback: T,
) -> Result<ClosureHookPoint<'a>, HookError> {
    let callback = move |reg: *mut Registers, original| {
        let original: extern "win64" fn(&EquipGameData, &mut DLMemoryOutputStream) -> usize =
            unsafe { std::mem::transmute(original) };
        // Safety: We trust that DS3 gives us valid pointers.
        let this = unsafe { &*((*reg).rcx as *const EquipGameData) };
        let stream = unsafe { &mut *((*reg).rdx as *mut DLMemoryOutputStream) };

        // Never write custom save data for the main menu.
        if !this.is_main_menu()
            && let Some(result) = callback()
        {
            // Add a small header indicating that fromsoftware-rs modified
            // this save file, so that we know which save files to run
            // [on_load] for.
            write!(stream, "{}", HEADER).unwrap();
            if stream.write_delimited(result.as_ref()).unwrap() != result.len() + 4 {
                return 1;
            }
        }

        original(this, stream)
    };

    unsafe {
        hook_closure_retn(
            *EQUIP_GAME_DATA_SERIALIZE_VA as usize,
            callback,
            CallbackOption::None,
            HookFlags::empty(),
        )
    }
}
