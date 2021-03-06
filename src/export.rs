/// Exports a given list of functions to a Erlang module.
///
/// This should be called exactly once in every NIF library. It will wrap and export the given rust
/// functions into the Erlang module.
///
/// The first argument is a string specifying what Erlang/Elixir module you want the function
/// exported into. In Erlang this will simply be the atom you named your module. In Elixir, all
/// modules are prefixed with `Elixir.<module path>`
///
/// The second argument is a list of 3-tuples. Each tuple contains information on a single exported
/// NIF function. The first tuple item is the name you want to export the function into, the second
/// is the arity (number of arguments) of the exported function. The third argument is a
/// indentifier of a rust function. This is where your actual NIF will be implemented.
///
/// The third argument is an `Option<fn(env: &NifEnv, load_info: NifTerm) -> bool>`. If this is
/// `Some`, the function will execute when the NIF is first loaded by the BEAM.
#[macro_export]
macro_rules! rustler_export_nifs {
    // Strip trailing comma.
    ($name:expr, [$( $exported_nif:tt ),+,], $on_load:expr) => {
        rustler_export_nifs!($name, [$( $exported_nif ),*], $on_load);
    };
    ($name:expr, [$( $exported_nif:tt ),*], $on_load:expr) => {
        static mut NIF_ENTRY: Option<$crate::codegen_runtime::DEF_NIF_ENTRY> = None;

        #[no_mangle]
        pub extern "C" fn nif_init() -> *const $crate::codegen_runtime::DEF_NIF_ENTRY {
            // TODO: If an unwrap ever happens, we will unwind right into C! Fix this!

            extern "C" fn nif_load(
                env: $crate::codegen_runtime::NIF_ENV,
                _priv_data: *mut *mut $crate::codegen_runtime::c_void,
                load_info: $crate::codegen_runtime::NIF_TERM)
                -> $crate::codegen_runtime::c_int {
                unsafe {
                    $crate::codegen_runtime::handle_nif_init_call($on_load, env, load_info)
                }
            }

            const FUN_ENTRIES: &'static [$crate::codegen_runtime::DEF_NIF_FUNC] = &[
                $(rustler_export_nifs!(internal, $exported_nif)),*
            ];

            let entry = $crate::codegen_runtime::DEF_NIF_ENTRY {
                major: $crate::codegen_runtime::NIF_MAJOR_VERSION,
                minor: $crate::codegen_runtime::NIF_MINOR_VERSION,
                name: concat!($name, "\x00") as *const str as *const u8,
                num_of_funcs: FUN_ENTRIES.len() as $crate::codegen_runtime::c_int,
                funcs: FUN_ENTRIES.as_ptr(),
                load: Some(nif_load),
                reload: None,
                upgrade: None,
                unload: None,
                vm_variant: b"beam.vanilla\x00".as_ptr(),
                options: 0,
            };
            unsafe { NIF_ENTRY = Some(entry) };

            unsafe { NIF_ENTRY.as_ref().unwrap() }
        }
    };

    (internal, ($nif_name:expr, $nif_arity:expr, $nif_fun:path)) => {
        rustler_export_nifs!(internal, ($nif_name, $nif_arity, $nif_fun, $crate::schedule::NifScheduleFlags::Normal))
    };
    (internal, ($nif_name:expr, $nif_arity:expr, $nif_fun:path, $nif_flag:expr)) => {
        $crate::codegen_runtime::DEF_NIF_FUNC {
            name: concat!($nif_name, "\x00") as *const str as *const u8,
            arity: $nif_arity,
            function: {
                extern "C" fn nif_func(
                    env: $crate::codegen_runtime::NIF_ENV,
                    argc: $crate::codegen_runtime::c_int,
                    argv: *const $crate::codegen_runtime::NIF_TERM)
                    -> $crate::codegen_runtime::NIF_TERM {
                    unsafe {
                        $crate::codegen_runtime::handle_nif_call($nif_fun, $nif_arity, env, argc, argv)
                    }
                }
                nif_func
            },
            flags: $nif_flag as u32,
        }
    };
}
