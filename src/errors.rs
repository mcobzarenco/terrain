error_chain! {
    // Define additional `ErrorKind` variants. The syntax here is
    // the same as `quick_error!`, but the `from()` and `cause()`
    // syntax is not supported.
    errors {
        LoadAssetError(msg: String) {
            description("Asset load error.")
            display("Asset load error: '{}'", msg)
        }
    }
}
