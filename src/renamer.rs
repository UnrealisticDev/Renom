use ini::Ini;
use regex::Regex;
use serde_json::Value;
use std::error::Error;
use std::fs::{self};
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};

use crate::pathfinder::Pathfinder;
use crate::printer::Print;

enum ProjectType {
    Blueprint,
    Code,
}

pub struct Renamer {
    proj_root: PathBuf,
    proj_original_name: String,
    proj_final_name: String,
    proj_type: ProjectType,
    pathfinder: Pathfinder,
}

impl Default for Renamer {
    fn default() -> Renamer {
        Renamer {
            proj_root: PathBuf::default(),
            proj_original_name: String::default(),
            proj_final_name: String::default(),
            proj_type: ProjectType::Blueprint,
            pathfinder: Pathfinder {
                root: PathBuf::default(),
            },
        }
    }
}

impl Renamer {
    /// Create a new project renamer.
    pub fn new() -> Renamer {
        Default::default()
    }

    /// Start the renaming process.
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        Print::check_support_for_colors();
        Print::header("Welcome to Renom");
        Print::header("Project Details");
        Print::basic("Tell us a little about your project.");
        self.request_project_root()?;
        self.infer_original_project_name()?;
        self.request_final_project_name()?;
        self.detect_project_type();

        Print::header("Backup & Staging");
        Print::basic("Setting up backup and staging directories.");
        self.backup_project_files()?;
        self.stage_project_files()?;

        Print::header("Rename");
        Print::basic("Renaming and updating project files.");
        self.update_project_descriptor()?;
        self.update_engine_config()?;
        if let ProjectType::Code = self.proj_type {
            self.update_target_files()?;
            self.update_build_file()?;
            self.update_module_source_files()?;
        }

        Print::header("Apply");
        Print::basic("So far, so good. Applying changes to live directory.");
        self.apply()?;
        self.rename_source_subfolder()?;
        self.rename_project_root()?;

        Print::header("Cleanup");
        if let ProjectType::Code = self.proj_type {
            if self.request_cleanup()? {
                self.cleanup()?;
            } else {
                Print::basic("Cleanup skipped.");
            }
        } else {
            Print::basic("Nothing to clean up for Blueprint project.");
        }

        Print::header("Success");
        Print::basic("Project successfully renamed.");
        Print::newline();

        Ok(())
    }

    /// Request the project root directory from the user.
    fn request_project_root(&mut self) -> Result<(), Box<dyn Error>> {
        Print::prompt("Project root");

        let mut root = String::new();
        match stdin().read_line(&mut root) {
            Ok(_) => {
                root = root.replace("\r", "").replace("\n", "");
            }
            Err(_) => return Err("No input provided.".to_owned())?,
        }

        let root = Path::new(&root);
        if !root.is_dir() {
            return Err("Provided path is not a directory.".to_owned())?;
        }

        self.proj_root = root.to_owned();
        self.pathfinder.root = root.to_owned();

        Ok(())
    }

    /// Infer the project's original name from the project root.
    fn infer_original_project_name(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.pathfinder.root_dir().is_dir() {
            return Err("Passed non-directory path as root.".to_owned())?;
        }

        let mut project_file: Option<PathBuf> = None;
        for entry in fs::read_dir(&mut self.pathfinder.root_dir())? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "uproject" {
                    project_file = Some(path);
                    break;
                }
            }
        }

        if let Some(path) = project_file {
            self.proj_original_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
            Print::basic(format!(
                "Project original name: {}",
                self.proj_original_name
            ));
            Ok(())
        } else {
            Err("Failed to find .uproject file in root directory.".to_owned())?
        }
    }

    /// Request final project name from user.
    fn request_final_project_name(&mut self) -> Result<(), String> {
        Print::prompt("Project final name");

        let mut final_name = String::new();
        match stdin().read_line(&mut final_name) {
            Ok(_) => {
                final_name = final_name.replace("\r", "").replace("\n", "");
            }
            _ => {}
        }

        if final_name.len() > 20 {
            return Err("Name is too long.".to_owned());
        }

        if final_name == self.proj_original_name {
            return Err("Final name is identical to original name.".to_owned());
        }

        self.proj_final_name = final_name;

        Ok(())
    }

    /// Detect project type (Blueprint or C++) based on existence of
    /// *Source* directory.
    fn detect_project_type(&mut self) {
        if self.pathfinder.source_dir().is_dir() {
            Print::basic("Code project detected.");
            self.proj_type = ProjectType::Code;
        } else {
            Print::basic("Blueprint project detected.");
            self.proj_type = ProjectType::Blueprint;
        }
    }

    /// Back up all relevant files.
    fn backup_project_files(&self) -> Result<(), Box<dyn Error>> {
        Print::process(format!(
            "Backing up files to {}",
            self.pathfinder
                .backup_dir()
                .to_str()
                .expect("Invalid backup directory path.")
        ));

        let backup_dir = self.pathfinder.backup_dir();
        if backup_dir.exists() {
            for entry in backup_dir.read_dir()? {
                fs::remove_file(entry.unwrap().path())?;
            }
        } else {
            fs::create_dir_all(&backup_dir)?;
        }

        self.backup_file(
            "Project Descriptor",
            self.pathfinder
                .root_dir()
                .join(&self.proj_original_name)
                .with_extension("uproject"),
        )?;
        self.backup_file(
            "Engine Config",
            self.pathfinder.config_dir().join("DefaultEngine.ini"),
        )?;

        if let ProjectType::Code = self.proj_type {
            self.backup_file(
                "Executable Target",
                self.pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .with_extension("Target.cs"),
            )?;
            self.backup_file(
                "Editor Target",
                self.pathfinder
                    .source_dir()
                    .join(format!("{}{}", &self.proj_original_name, "Editor"))
                    .with_extension("Target.cs"),
            )?;
            self.backup_file(
                "Primary Game Module Build File",
                self.pathfinder
                    .source_proj_dir(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("Build.cs"),
            )?;
            self.backup_file(
                "Primary Game Module Header",
                self.pathfinder
                    .source_proj_dir(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("h"),
            )?;
            self.backup_file(
                "Primary Game Module Source File",
                self.pathfinder
                    .source_proj_dir(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("cpp"),
            )?;

            fn backup_api_referencers_in_dir<P: AsRef<Path>>(
                renamer: &Renamer,
                dir: P,
            ) -> Result<(), Box<dyn Error>> {
                for entry in dir.as_ref().read_dir()? {
                    let entry = entry.unwrap().path();
                    if let Some(ext) = entry.extension() {
                        if ext == "h" {
                            let mut file = fs::File::open(&entry).unwrap();
                            let mut data = String::new();
                            file.read_to_string(&mut data).unwrap();
                            if data.contains(
                                format!("{}_API", renamer.proj_original_name.to_uppercase())
                                    .as_str(),
                            ) {
                                renamer.backup_file(
                                    entry.file_name().unwrap().to_str().unwrap(),
                                    &entry,
                                )?;
                            }
                        }
                    } else {
                        backup_api_referencers_in_dir(renamer, entry)?;
                    }
                }
                Ok(())
            };

            backup_api_referencers_in_dir(&self, self.pathfinder.source_dir())?;
        }

        Ok(())
    }

    /// Back up the specified file.
    fn backup_file<P: AsRef<Path>>(
        &self,
        description: &str,
        file: P,
    ) -> Result<(), Box<dyn Error>> {
        Print::step("Backing up", description);

        let file = file.as_ref();

        if !file.is_file() {
            return Err(format!(
                "Failed to find file: {:?}. Unable to backup.",
                file
            ))?;
        }

        if !self.pathfinder.backup_dir().exists() {
            return Err(format!(
                "Backup directory does not exist. It may have been deleted at some point."
            ))?;
        }

        let file_name = file.file_name().unwrap();
        let destination = self.pathfinder.backup_dir().join(file_name);
        fs::copy(file, destination)?;

        Ok(())
    }

    /// Stage all relevant files. These files will be worked on
    /// and, if all goes well, eventually copied to the live directory.
    fn stage_project_files(&self) -> Result<(), Box<dyn Error>> {
        Print::process(format!(
            "Staging files to {}.",
            self.pathfinder
                .staging_dir()
                .to_str()
                .expect("Invalid staging directory path.")
        ));

        let staging_dir = self.pathfinder.staging_dir();
        if staging_dir.exists() {
            for entry in staging_dir.read_dir()? {
                fs::remove_file(entry.unwrap().path())?;
            }
        } else {
            fs::create_dir_all(&staging_dir)?;
        }

        let backup_dir = &self.pathfinder.backup_dir();
        for entry in backup_dir.read_dir()? {
            let entry = entry.unwrap().path();
            Print::step("Staging", entry.file_name().unwrap().to_str().unwrap());
            fs::copy(
                &entry,
                &staging_dir.join(&entry.file_name().unwrap().to_str().unwrap()),
            )
            .unwrap();
        }

        Ok(())
    }

    /// Rename and update project descriptor (.uproject).
    fn update_project_descriptor(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Updating project descriptor.");
        let original = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_original_name)
            .with_extension("uproject");

        let _final = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_final_name)
            .with_extension("uproject");

        Print::step(
            "Update project descriptor",
            &format!("Renaming descriptor to {}", self.proj_final_name),
        );
        fs::rename(&original, &_final)?;

        if let ProjectType::Code = self.proj_type {
            Print::step(
                "Update project descriptor",
                "Replacing instances of old name with new name within descriptor.",
            );

            let mut file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .append(false)
                .create(true)
                .open(&_final)?;
            let mut data = String::new();
            file.read_to_string(&mut data)?;

            let mut v: Value = serde_json::from_str(&data)?;
            for module in v["Modules"].as_array_mut().unwrap() {
                if let Value::String(name) = &mut module["Name"] {
                    if name == &self.proj_original_name {
                        *name = self.proj_final_name.clone();
                        break;
                    }
                }
            }

            fs::write(&_final, v.to_string())?;
        }

        Ok(())
    }

    /// Update engine config file (DefaultEngine.ini).
    fn update_engine_config(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Updating engine config file.");

        let config_path = &self.pathfinder.staging_dir().join("DefaultEngine.ini");
        Print::step(
            "Update engine config",
            &format!("Opening config file at {:?}", config_path),
        );
        let mut config = Ini::load_from_file(&config_path)?;

        Print::step(
            "Update engine config",
            "Adding GameName entry under [URL] header to DefaultEngine.ini.",
        );
        config
            .with_section(Some("URL"))
            .set("GameName", &self.proj_final_name);

        if let ProjectType::Code = self.proj_type {
            Print::step(
                "Update engine config",
                "Adding ActiveGameNameRedirect entry(s) under [/Script/Engine.Engine] header to DefaultEngine.ini."
            );

            let redir_header = Some("/Script/Engine.Engine");
            let redir_key = "+ActiveGameNameRedirects";

            // Note we have to create this dummy section
            // because `section_mut` fails if the section doesn't exist,
            // but `with_section` doesn't allow you to manipulate
            // keys with multiple values
            config
                .with_section(redir_header.clone())
                .set("Dummy", "dummy");

            let mut old_redirects = Vec::new();
            config
                .section_mut(redir_header.clone())
                .unwrap()
                .remove_all(redir_key)
                .for_each(|val| {
                    old_redirects.push(val);
                });

            let re = Regex::new("NewGameName=\"/Script/(.+)\"").unwrap();
            for val in old_redirects {
                config.section_mut(redir_header.clone()).unwrap().append(
                    redir_key,
                    re.replace(
                        &val,
                        format!("NewGameName=\"/Script/{}\"", self.proj_final_name).as_str(),
                    ),
                );
            }

            config.section_mut(redir_header.clone()).unwrap().append(
                redir_key,
                format!(
                    "(OldGameName=\"/Script/{}\", NewGameName=\"/Script/{}\")",
                    &self.proj_original_name, &self.proj_final_name
                ),
            );
            config
                .section_mut(Some("/Script/Engine.Engine"))
                .unwrap()
                .remove("Dummy");
        }

        Print::step("Update engine config", "Writing to file.");
        config.write_to_file(config_path)?;

        Ok(())
    }

    /// Rename and update the project target files.
    fn update_target_files(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Updating target files.");
        let original = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_original_name)
            .with_extension("Target.cs");
        let _final = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_final_name)
            .with_extension("Target.cs");

        Print::step(
            "Update exec target",
            &format!("Renaming target file to: {}", &self.proj_final_name),
        );
        fs::rename(&original, &_final)?;

        Print::step(
            "Update exec target",
            "Replacing instances of old project name within file.",
        );
        let mut file = fs::File::open(&_final)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        fs::write(
            &_final,
            data.replace(&self.proj_original_name, &self.proj_final_name),
        )?;

        let original = &self
            .pathfinder
            .staging_dir()
            .join(format!("{}{}", &self.proj_original_name, "Editor"))
            .with_extension("Target.cs");
        let _final = &self
            .pathfinder
            .staging_dir()
            .join(format!("{}{}", &self.proj_final_name, "Editor"))
            .with_extension("Target.cs");

        Print::step(
            "Update editor target",
            &format!("Renaming target file to: {}", &self.proj_final_name),
        );
        fs::rename(&original, &_final).unwrap();

        Print::step(
            "Update editor target",
            "Replacing instances of old project name within file.",
        );
        let mut file = fs::File::open(&_final)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        fs::write(
            &_final,
            data.replace(&self.proj_original_name, &self.proj_final_name),
        )?;

        Ok(())
    }

    /// Update the project primary game mode build file.
    fn update_build_file(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Updating primary game module build file.");
        let original = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_original_name)
            .with_extension("Build.cs");
        let _final = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_final_name)
            .with_extension("Build.cs");

        Print::step("Update build file", "Renaming file.");
        fs::rename(&original, &_final)?;

        Print::step(
            "Update build file",
            "Replacing old project name instances within file.",
        );
        let mut file = fs::File::open(&_final)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        fs::write(
            &_final,
            data.replace(&self.proj_original_name, &self.proj_final_name),
        )?;

        Ok(())
    }

    /// Update project primary game module source files.
    fn update_module_source_files(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Updating primary game module source files.");

        let original = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_original_name)
            .with_extension("h");

        let _final = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_final_name)
            .with_extension("h");

        Print::step("Update module files", "Renaming header file.");
        fs::rename(&original, &_final)?;

        Print::step(
            "Update module files",
            "Replacing instances of old project name within header file.",
        );
        let mut file = fs::File::open(&_final)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        fs::write(
            &_final,
            data.replace(&self.proj_original_name, &self.proj_final_name),
        )?;

        let original = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_original_name)
            .with_extension("cpp");

        let _final = &self
            .pathfinder
            .staging_dir()
            .join(&self.proj_final_name)
            .with_extension("cpp");

        Print::step("Update module files", "Renaming source file.");
        fs::rename(&original, &_final)?;

        Print::step(
            "Update module files",
            "Replacing instances of old project name within source file.",
        );
        let mut file = fs::File::open(&_final)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        fs::write(
            &_final,
            data.replace(&self.proj_original_name, &self.proj_final_name),
        )?;

        Ok(())
    }

    /// Update source file PROJECT_API references
    fn _update_api_references(&self) {
        for entry in self.pathfinder.staging_dir().read_dir().unwrap() {
            let entry = entry.unwrap().path();
            if let Some(ext) = entry.extension() {
                if ext == "h" {
                    let mut file = fs::File::open(&entry).unwrap();
                    let mut data = String::new();
                    file.read_to_string(&mut data).unwrap();
                    fs::write(
                        &entry,
                        data.replace(
                            format!("{}_API", &self.proj_original_name.to_uppercase()).as_str(),
                            format!("{}_API", &self.proj_final_name.to_uppercase()).as_str(),
                        ),
                    )
                    .unwrap();
                    println!(
                        "Replacing outdated api reference in source file: {:?}",
                        &entry.file_name()
                    );
                }
            }
        }
    }

    /// Apply staged changes to live directory.
    fn apply(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Emplacing live directory files with staged files.");
        Renamer::emplace_file(
            "Project Descriptor",
            &self
                .pathfinder
                .root_dir()
                .join(&self.proj_original_name)
                .with_extension("uproject"),
            &self
                .pathfinder
                .staging_dir()
                .join(&self.proj_final_name)
                .with_extension("uproject"),
            &self
                .pathfinder
                .root_dir()
                .join(&self.proj_final_name)
                .with_extension("uproject"),
        )?;

        Renamer::emplace_file(
            "Engine Config",
            &self
                .pathfinder
                .root_dir()
                .join("Config")
                .join("DefaultEngine.ini"),
            &self.pathfinder.staging_dir().join("DefaultEngine.ini"),
            &self
                .pathfinder
                .root_dir()
                .join("Config")
                .join("DefaultEngine.ini"),
        )?;

        if let ProjectType::Code = self.proj_type {
            Renamer::emplace_file(
                "Executable Target",
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .with_extension("Target.cs"),
                &self
                    .pathfinder
                    .staging_dir()
                    .join(&self.proj_final_name)
                    .with_extension("Target.cs"),
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_final_name)
                    .with_extension("Target.cs"),
            )?;

            Renamer::emplace_file(
                "Editor Target",
                &self
                    .pathfinder
                    .source_dir()
                    .join(format!("{}Editor", &self.proj_original_name))
                    .with_extension("Target.cs"),
                &self
                    .pathfinder
                    .staging_dir()
                    .join(format!("{}Editor", &self.proj_final_name))
                    .with_extension("Target.cs"),
                &self
                    .pathfinder
                    .source_dir()
                    .join(format!("{}Editor", &self.proj_final_name))
                    .with_extension("Target.cs"),
            )?;

            Renamer::emplace_file(
                "Module Build File",
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("Build.cs"),
                &self
                    .pathfinder
                    .staging_dir()
                    .join(&self.proj_final_name)
                    .with_extension("Build.cs"),
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_final_name)
                    .with_extension("Build.cs"),
            )?;

            Renamer::emplace_file(
                "Module Header File",
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("h"),
                &self
                    .pathfinder
                    .staging_dir()
                    .join(&self.proj_final_name)
                    .with_extension("h"),
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_final_name)
                    .with_extension("h"),
            )?;

            Renamer::emplace_file(
                "Module Source File",
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_original_name)
                    .with_extension("cpp"),
                &self
                    .pathfinder
                    .staging_dir()
                    .join(&self.proj_final_name)
                    .with_extension("cpp"),
                &self
                    .pathfinder
                    .source_dir()
                    .join(&self.proj_original_name)
                    .join(&self.proj_final_name)
                    .with_extension("cpp"),
            )?;

            fn replace_api_referencers<P: AsRef<Path>>(
                renamer: &Renamer,
                dir: P,
            ) -> Result<(), Box<dyn Error>> {
                for entry in dir.as_ref().read_dir()? {
                    let entry = entry.unwrap().path();
                    if let Some(ext) = entry.extension() {
                        if ext == "h" {
                            let mut file = fs::File::open(&entry)?;
                            let mut data = String::new();
                            file.read_to_string(&mut data)?;
                            if data.contains(
                                format!("{}_API", renamer.proj_original_name.to_uppercase())
                                    .as_str(),
                            ) {
                                Print::step(
                                    "Emplace",
                                    &format!(
                                        "Replacing outdated _API references in file: {}.",
                                        entry.to_str().unwrap()
                                    ),
                                );
                                fs::write(
                                    &entry,
                                    data.replace(
                                        format!(
                                            "{}_API",
                                            renamer.proj_original_name.to_uppercase()
                                        )
                                        .as_str(),
                                        format!("{}_API", renamer.proj_final_name.to_uppercase())
                                            .as_str(),
                                    ),
                                )?;
                            }
                        }
                    } else {
                        replace_api_referencers(renamer, entry)?;
                    }
                }

                Ok(())
            };
            replace_api_referencers(&self, self.pathfinder.source_dir())?;
        }

        Ok(())
    }

    /// Apply changes to live directory for file with `description` by:
    /// * Deleting `original`
    /// * Copying `staged` to `_final`
    fn emplace_file<P: AsRef<Path>>(
        description: &str,
        original: P,
        staged: P,
        _final: P,
    ) -> Result<(), Box<dyn Error>> {
        Print::step("Emplace", &format!("Emplacing {}", description));
        fs::remove_file(&original)
            .expect(format!("Failed to delete original {}.", description).as_str());
        fs::copy(&staged, &_final)
            .expect(format!("Failed to copy {} from staged to final.", description).as_str());

        Ok(())
    }

    /// Rename the project Source/{project_name} subfolder.
    fn rename_source_subfolder(&self) -> Result<(), Box<dyn Error>> {
        if let ProjectType::Code = self.proj_type {
            Print::process("Renaming source project subfolder");
            let original = &self.pathfinder.source_proj_dir(&self.proj_original_name);
            let _final = &self.pathfinder.source_proj_dir(&self.proj_final_name);
            Print::step("Source subfolder rename", "Renaming");
            fs::rename(&original, &_final)?;
        }
        Ok(())
    }
    /// Rename the project root folder.
    fn rename_project_root(&mut self) -> Result<(), Box<dyn Error>> {
        Print::process("Renaming root folder");
        let original = &self.proj_root;
        let _final = self.proj_root.with_file_name(&self.proj_final_name);
        Print::step("Root folder rename", "Renaming");
        fs::rename(&original, &_final)?;
        self.proj_root = _final;

        Ok(())
    }

    /// Request cleanup.
    fn request_cleanup(&self) -> Result<bool, Box<dyn Error>> {
        Print::basic("Though not strictly necessary, it is a good idea to clean up outdated Saved, Intermediate, and Binaries folders.\nShall we go ahead and do so for you?");
        Print::prompt("[Y]es/[N]o");

        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        if let Some(c) = buf.chars().next() {
            if c == 'y' || c == 'Y' {
                return Ok(true);
            } else {
                return Ok(false);
            }
        } else {
            Err("No input provided.")?
        }

        Ok(true)
    }

    /// Cleanup *Saved*, *Intermediate*, and *Binaries* directories.
    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        Print::process("Cleaning up outdated directories.");

        Print::step("Cleanup", "Deleting Saved directory.");
        let saved_dir = self.proj_root.join("Saved");
        if saved_dir.is_dir() {
            fs::remove_dir_all(saved_dir)?;
        } else {
            Print::step("Cleanup", "Does not exist. Skipped.");
        }

        Print::step("Cleanup", "Deleting Intermediate directory.");
        let intermediate_dir = self.proj_root.join("Intermediate");
        if intermediate_dir.is_dir() {
            fs::remove_dir_all(intermediate_dir)?;
        } else {
            Print::step("Cleanup", "Does not exist. Skipped.");
        }

        Print::step("Cleanup", "Deleting Binaries directory.");
        let binaries_dir = self.proj_root.join("Binaries");
        if binaries_dir.is_dir() {
            fs::remove_dir_all(binaries_dir)?;
        } else {
            Print::step("Cleanup", "Does not exist. Skipped.");
        }

        Ok(())
    }
}
