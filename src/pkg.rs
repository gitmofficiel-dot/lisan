//! Package Manager for Lisan (`lisan pkg` / `lisan حزمة`).
//! Provides project initialization, dependency registration, and lockfile generation.

use std::fs;
use std::path::{Path, PathBuf};

/// Package manifest schema representation for `lisan.json`
#[derive(Debug, Clone, PartialEq)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub dependencies: Vec<(String, String)>, // (name, path)
}

impl PackageManifest {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("مشروع لسان: {}", name),
            author: "مطور لسان".to_string(),
            license: "MIT".to_string(),
            dependencies: Vec::new(),
        }
    }

    pub fn to_json(&self) -> String {
        let mut deps_entries = Vec::new();
        for (k, v) in &self.dependencies {
            deps_entries.push(format!("    \"{}\": \"{}\"", k, v));
        }
        let deps_str = if deps_entries.is_empty() {
            "{}".to_string()
        } else {
            format!("{{\n{}\n  }}", deps_entries.join(",\n"))
        };

        format!(
            "{{\n  \"اسم\": \"{}\",\n  \"إصدار\": \"{}\",\n  \"وصف\": \"{}\",\n  \"مؤلف\": \"{}\",\n  \"رخصة\": \"{}\",\n  \"اعتماديات\": {}\n}}\n",
            self.name, self.version, self.description, self.author, self.license, deps_str
        )
    }

    pub fn from_json(content: &str) -> Result<Self, String> {
        let parsed = crate::json::parse_json(content)
            .map_err(|e| format!("خطأ في قراءة ملف حزمة lisan.json: {}", e))?;

        let mut manifest = PackageManifest::new("مشروع_لسان");

        if let crate::runtime::Value::Map(pairs) = parsed {
            for (key, val) in pairs.borrow().iter() {
                match key.as_str() {
                    "اسم" | "name" => {
                        if let crate::runtime::Value::Text(s) = val { manifest.name = s.clone(); }
                    }
                    "إصدار" | "version" => {
                        if let crate::runtime::Value::Text(s) = val { manifest.version = s.clone(); }
                        else if let crate::runtime::Value::Number(n) = val { manifest.version = format!("{}", n); }
                    }
                    "وصف" | "description" => {
                        if let crate::runtime::Value::Text(s) = val { manifest.description = s.clone(); }
                    }
                    "مؤلف" | "author" => {
                        if let crate::runtime::Value::Text(s) = val { manifest.author = s.clone(); }
                    }
                    "رخصة" | "license" => {
                        if let crate::runtime::Value::Text(s) = val { manifest.license = s.clone(); }
                    }
                    "اعتماديات" | "dependencies" => {
                        if let crate::runtime::Value::Map(dep_pairs) = val {
                            for (dk, dv) in dep_pairs.borrow().iter() {
                                let dep_val = match dv {
                                    crate::runtime::Value::Text(ts) => ts.clone(),
                                    other => format!("{:?}", other),
                                };
                                manifest.dependencies.push((dk.clone(), dep_val));
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            return Err("ملف الحزمة يجب أن يكون كائن JSON".into());
        }

        Ok(manifest)
    }
}

/// Initialize a new Lisan package in the target directory.
pub fn init_project(name: &str, dir: &Path) -> Result<PathBuf, String> {
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| format!("تعذر إنشاء المجلد: {}", e))?;
    }

    let manifest_path = dir.join("lisan.json");
    if manifest_path.exists() {
        return Err(format!("ملف lisan.json موجود مسبقاً في: {}", dir.display()));
    }

    let manifest = PackageManifest::new(name);
    fs::write(&manifest_path, manifest.to_json())
        .map_err(|e| format!("تعذر حفظ lisan.json: {}", e))?;

    let lib_dir = dir.join("مكتبات");
    if !lib_dir.exists() {
        let _ = fs::create_dir_all(&lib_dir);
    }

    let main_file = dir.join("رئيسي.lsn");
    if !main_file.exists() {
        let default_code = format!(
            "# مشروع {}\nاطبع(\"مرحباً بك في لغة لسان!\")\n",
            name
        );
        let _ = fs::write(&main_file, default_code);
    }

    // Generate initial lockfile
    let _ = create_lockfile(dir);

    Ok(manifest_path)
}

/// Add a dependency to `lisan.json` and refresh `lisan.lock`.
pub fn add_dependency(dep_name: &str, path_or_src: &str, dir: &Path) -> Result<(), String> {
    let manifest_path = dir.join("lisan.json");
    if !manifest_path.exists() {
        return Err("لم يتم العثور على ملف lisan.json في المجلد الحالي; شغّل `lisan --pkg init` أولاً".into());
    }

    let content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("تعذرت قراءة lisan.json: {}", e))?;
    let mut manifest = PackageManifest::from_json(&content)?;

    manifest.dependencies.retain(|(k, _)| k != dep_name);
    manifest.dependencies.push((dep_name.to_string(), path_or_src.to_string()));

    fs::write(&manifest_path, manifest.to_json())
        .map_err(|e| format!("تعذر تحديث lisan.json: {}", e))?;

    create_lockfile(dir)?;
    Ok(())
}

/// Create or update `lisan.lock` ensuring reproducible dependency status.
pub fn create_lockfile(dir: &Path) -> Result<PathBuf, String> {
    let manifest_path = dir.join("lisan.json");
    let lock_path = dir.join("lisan.lock");

    let manifest = if manifest_path.exists() {
        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("تعذرت قراءة lisan.json: {}", e))?;
        PackageManifest::from_json(&content)?
    } else {
        PackageManifest::new("مشروع")
    };

    let mut lock_content = String::new();
    lock_content.push_str("# ملف قفل حزمة لسان (Lisan Lockfile) - يتم توليده تلقائياً ولا يُعدّل يدوياً\n");
    lock_content.push_str(&format!("الحزمة = \"{}\"\n", manifest.name));
    lock_content.push_str(&format!("الإصدار = \"{}\"\n\n", manifest.version));

    lock_content.push_str("[[الاعتماديات]]\n");
    for (name, path) in &manifest.dependencies {
        lock_content.push_str(&format!("اسم = \"{}\"\n", name));
        lock_content.push_str(&format!("المسار = \"{}\"\n", path));
        let dep_path = dir.join(path);
        let status = if dep_path.exists() { "مثبت" } else { "مسار_خارجي" };
        lock_content.push_str(&format!("الحالة = \"{}\"\n\n", status));
    }

    fs::write(&lock_path, lock_content)
        .map_err(|e| format!("تعذر حفظ lisan.lock: {}", e))?;

    Ok(lock_path)
}
