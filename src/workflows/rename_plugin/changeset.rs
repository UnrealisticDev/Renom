use std::path::Path;

use crate::{
    changes::{AppendIniEntry, Change, RenameFile, ReplaceInFile},
    unreal::Plugin,
};

use super::Context;

/// Generate a changeset to rename an Unreal Engine plugin.
pub fn generate_changeset(context: &Context) -> Vec<Change> {
    let Context {
        project_root,
        project_name,
        project_plugins,
        plugin: Plugin {
            name: old_name,
            root: plugin_root,
        },
        new_name,
    } = context;

    let descriptor = plugin_root.join(old_name).with_extension("uplugin");
    let mut changeset = vec![];

    changeset.push(rename_plugin_descriptor(&descriptor, new_name));
    changeset.push(rename_plugin_root(plugin_root, new_name));
    changeset.push(rename_plugin_reference_in_project_descriptor(
        project_root,
        project_name,
        old_name,
        new_name,
    ));
    changeset.extend(rename_cross_plugin_references(
        project_plugins,
        old_name,
        new_name,
    ));
    changeset.push(update_existing_redirects(project_root, old_name, new_name));
    changeset.push(append_plugin_redirect(project_root, old_name, new_name));

    changeset
}

fn rename_plugin_descriptor(descriptor: &Path, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(
        descriptor,
        descriptor.with_file_name(format!("{new_name}.uplugin")),
    ))
}

fn rename_plugin_root(root: &Path, new_name: &str) -> Change {
    Change::RenameFile(RenameFile::new(root, root.with_file_name(new_name)))
}

fn rename_plugin_reference_in_project_descriptor(
    root: &Path,
    project_name: &str,
    old_name: &str,
    new_name: &str,
) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        root.join(project_name).with_extension("uproject"),
        format!(r#""{old_name}""#),
        format!(r#""{new_name}""#),
    ))
}

fn rename_cross_plugin_references(
    project_plugins: &[Plugin],
    old_name: &str,
    new_name: &str,
) -> Vec<Change> {
    project_plugins
        .iter()
        .filter(|plugin| &plugin.name != old_name)
        .map(|plugin| rename_plugin_references_in_plugin(&plugin, old_name, new_name))
        .collect()
}

fn rename_plugin_references_in_plugin(plugin: &Plugin, old_name: &str, new_name: &str) -> Change {
    let plugin_descriptor = plugin.root.join(&plugin.name).with_extension("uplugin");
    Change::ReplaceInFile(ReplaceInFile::new(
        plugin_descriptor,
        format!(r#""{old_name}""#),
        format!(r#""{new_name}""#),
    ))
}

fn update_existing_redirects(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::ReplaceInFile(ReplaceInFile::new(
        project_root.join("Config").join("DefaultEngine.ini"),
        format!(
            r#"\(OldName="/(?P<old>.+?)/",\s*NewName="/{}/",\s*MatchSubstring=true\)"#,
            old_name
        ),
        format!(
            r#"(OldName="/$old/",NewName="/{}/",MatchSubstring=true)"#,
            new_name
        ),
    ))
}

fn append_plugin_redirect(project_root: &Path, old_name: &str, new_name: &str) -> Change {
    Change::AppendIniEntry(AppendIniEntry::new(
        project_root.join("Config").join("DefaultEngine.ini"),
        "CoreRedirects",
        "+PackageRedirects",
        format!(
            r#"(OldName="/{}/",NewName="/{}/",MatchSubstring=true)"#,
            old_name, new_name
        ),
    ))
}
