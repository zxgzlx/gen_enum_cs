mod code;
use code::gen_code;

#[no_mangle]
pub extern "C" fn export_gen_cs_code() {
    gen_code();
}
