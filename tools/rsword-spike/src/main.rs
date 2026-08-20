//! Isolated spike: prove whether rsword_chirho can read the CrossWire
//! KJVA (KJV + Apocrypha) and LXX (Septuagint) zText modules.
//!
//! Run: cargo run --bin spike_sword -- <sword-root>

use std::time::Instant;

use rsword_chirho::manager_chirho::SwMgrChirho;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| "spike/sword".into());
    let t0 = Instant::now();
    let mut mgr = SwMgrChirho::new_chirho();
    mgr.add_path_chirho(&root);
    if let Err(e) = mgr.load_modules_chirho() {
        eprintln!("load_modules error: {e:?}");
        std::process::exit(1);
    }
    println!("modules loaded in {:?}: {:?}", t0.elapsed(), mgr.get_module_names_chirho());

    for name in ["KJVA", "LXX"] {
        let conf = match mgr.get_module_chirho(name) {
            Some(c) => c,
            None => {
                println!("[{name}] no module config found");
                continue;
            }
        };
        let conf_path = std::path::Path::new(&root).join("mods.d").join(format!("{}.conf", name.to_lowercase()));
        let conf2 = match rsword_chirho::config_chirho::ModuleConfigChirho::from_file_chirho(&conf_path) {
            Ok(c) => c,
            Err(e) => {
                println!("[{name}] conf parse error: {e:?}");
                continue;
            }
        };
        let v11n = conf2.versification_chirho();
        println!("[{name}] v11n in conf: {v11n:?} (registered: {:?})",
            rsword_chirho::versification_chirho::get_versification_chirho(&v11n).is_some());

        let loaded = match rsword_chirho::manager_chirho::module_factory_chirho::load_module_chirho(std::path::Path::new(&root), &conf2) {
            Ok(m) => m,
            Err(e) => {
                println!("[{name}] load_module error: {e:?}");
                continue;
            }
        };

        for probe in ["Sirach 2:1", "Tobit 1:1", "Genesis 1:1", "1 Maccabees 3:1"] {
            let t1 = Instant::now();
            let raw = loaded.read_entry_filtered_chirho(
                probe,
                rsword_chirho::manager_chirho::module_factory_chirho::OutputFormatChirho::PlainChirho,
                &rsword_chirho::FilterOptionsChirho::default(),
            );
            let display = raw.as_deref().map(|s| {
                let mut chars = s.chars();
                let head: String = chars.by_ref().take(140).collect();
                head
            });
            println!("[{name}] {probe}: {:?} ({:?})", display, t1.elapsed());
        }
    }
    probe_structure(&root, "KJVA");
    probe_structure(&root, "LXX");
}

#[allow(dead_code)]
fn probe_structure(root: &str, name: &str) {
    let conf_path = std::path::Path::new(root).join("mods.d").join(format!("{}.conf", name.to_lowercase()));
    let conf2 = match rsword_chirho::config_chirho::ModuleConfigChirho::from_file_chirho(&conf_path) {
        Ok(c) => c,
        Err(e) => { eprintln!("conf err {e:?}"); return; }
    };
    let loaded = match rsword_chirho::manager_chirho::module_factory_chirho::load_module_chirho(std::path::Path::new(root), &conf2) {
        Ok(m) => m,
        Err(e) => { eprintln!("load err {e:?}"); return; }
    };
    let roots = loaded.get_root_keys_chirho();
    println!("[{name}] root keys: {roots:?}");
    for r in roots.iter().take(120) {
        let kids = loaded.get_children_chirho(r);
        println!("[{name}] {r} -> {} children: {:?}", kids.len(), &kids[..kids.len().min(6)]);
    }
}
