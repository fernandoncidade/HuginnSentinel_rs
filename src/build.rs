fn main() {
    println!("cargo:rerun-if-changed=src/assets/images/icons/huginn.ico");
    println!("cargo:rerun-if-changed=src/build.rs");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/assets/images/icons/huginn.ico");
        res.set("FileDescription", "Huginn Sentinel");
        res.set("ProductName", "Huginn Sentinel");
        res.set("LegalCopyright", "Fernando Cidade");
        if let Err(e) = res.compile() {
            eprintln!("Erro ao compilar recurso Windows (.ico): {}", e);
            std::process::exit(1);
        }
    }
}

