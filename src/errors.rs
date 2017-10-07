error_chain! {
    // The type defined for this error. These are the conventional
    // and recommended names, but they can be arbitrarily chosen.
    // It is also possible to leave this block out entirely, or
    // leave it empty, and these names will be used automatically.
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    // Define additional `ErrorKind` variants. The syntax here is
    // the same as `quick_error!`, but the `from()` and `cause()`
    // syntax is not supported.
    errors {
        LoadAssetError(msg: String) {
            description("Asset load error.")
            display("Asset load error: '{}'", msg)
        }
        MissingGlutinWindow {
            description("The glutin window is missing.")
            display("The glutin window is missing.")
        }
        SetCursorPositionError(x: i32, y: i32) {
            description("Could not set cursor position.")
            display("Could not set cursor position to ({}, {})", x, y)
        }
        UnexhaustedHeightmapFile {
            description("More data than expected in heightmap file.")
            display("More data than expected in heightmap file.")
        }
    }
}
