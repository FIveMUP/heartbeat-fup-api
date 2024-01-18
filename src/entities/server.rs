use compact_str::CompactString;

#[derive(Debug)]
pub struct Server {
    pub id: CompactString,
    pub name: CompactString,
    pub cfx_license: CompactString,
    pub cfx_code: Option<CompactString>,
    pub sv_license_key_token: CompactString,
}
