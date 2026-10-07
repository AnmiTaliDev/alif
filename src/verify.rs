// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::checker::check;
use crate::context::{Context, Sequent};
use crate::error::{AlifError, LoadError};
use crate::parser::parse_source;
use crate::rules::Rule;
use crate::stdlib;
use crate::term::Item;

pub(crate) struct Verifier {
    ctx: Context,
    shadowable: HashSet<String>,
    loaded: HashSet<PathBuf>,
    active: Vec<PathBuf>,
}

impl Verifier {
    pub(crate) fn with_context(ctx: Context) -> Verifier {
        let shadowable = ctx
            .axioms
            .keys()
            .chain(ctx.theorems.keys())
            .cloned()
            .collect();
        Verifier {
            ctx,
            shadowable,
            loaded: HashSet::new(),
            active: Vec::new(),
        }
    }

    pub(crate) fn new() -> Result<Verifier, AlifError> {
        Ok(Verifier::with_context(stdlib::load_stdlib()?))
    }

    pub(crate) fn into_context(self) -> Context {
        self.ctx
    }

    fn reserve(&mut self, name: &str) -> Result<(), LoadError> {
        if Rule::from_name(name).is_some() {
            return Err(LoadError::new(format!(
                "`{}` is the name of a built-in rule",
                name
            )));
        }
        if self.ctx.contains(name) {
            if !self.shadowable.remove(name) {
                return Err(LoadError::new(format!("`{}` is already defined", name)));
            }
            self.ctx.axioms.remove(name);
            self.ctx.theorems.remove(name);
        }
        Ok(())
    }

    pub(crate) fn process(
        &mut self,
        source: &str,
        label: Option<&str>,
        dir: Option<&Path>,
    ) -> Result<(), AlifError> {
        let items = parse_source(source).map_err(|e| e.locate(label, source))?;
        for item in items {
            match item {
                Item::Axiom {
                    name,
                    formula,
                    offset,
                } => {
                    self.reserve(&name)
                        .map_err(|e| e.at(label, source, offset))?;
                    self.ctx.axioms.insert(name, formula);
                }
                Item::Theorem(theorem) => {
                    self.reserve(&theorem.name)
                        .map_err(|e| e.at(label, source, theorem.offset))?;
                    check(&theorem, &self.ctx).map_err(|e| e.locate(label, source))?;
                    self.ctx.theorems.insert(
                        theorem.name,
                        Sequent {
                            hypotheses: theorem.hypotheses,
                            conclusion: theorem.conclusion,
                        },
                    );
                }
                Item::Import { path, offset } => {
                    self.import(&path, dir).map_err(|e| match e {
                        AlifError::Load(load) if load.location.is_none() => {
                            AlifError::Load(load.at(label, source, offset))
                        }
                        other => other,
                    })?;
                }
            }
        }
        Ok(())
    }

    fn import(&mut self, path: &str, dir: Option<&Path>) -> Result<(), AlifError> {
        match dir {
            Some(dir) => self.load(&dir.join(path)),
            None => Err(LoadError::new(
                "`import` needs a file location; verify a file instead of a source string",
            )
            .into()),
        }
    }

    pub(crate) fn load(&mut self, path: &Path) -> Result<(), AlifError> {
        let unreadable = |e: std::io::Error| {
            LoadError::new(format!("cannot read `{}`: {}", path.display(), e))
        };
        let canonical = fs::canonicalize(path).map_err(unreadable)?;
        if self.active.contains(&canonical) {
            return Err(LoadError::new(format!(
                "import cycle through `{}`",
                path.display()
            ))
            .into());
        }
        if self.loaded.contains(&canonical) {
            return Ok(());
        }
        let source = fs::read_to_string(&canonical).map_err(unreadable)?;
        self.active.push(canonical.clone());
        let label = path.display().to_string();
        let result = self.process(&source, Some(&label), path.parent());
        self.active.pop();
        result?;
        self.loaded.insert(canonical);
        Ok(())
    }
}

pub fn verify_source(source: &str) -> Result<(), AlifError> {
    Verifier::new()?.process(source, None, None)
}

pub fn verify_file(path: &Path) -> Result<(), AlifError> {
    Verifier::new()?.load(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY: &str = "theorem id_thm: A |- A\nproof\n  assume h: A\n  exact h\nqed\n";

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("alif-test-{}-{}", std::process::id(), name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn source_with_valid_proof() {
        verify_source(IDENTITY).unwrap();
    }

    #[test]
    fn source_with_invalid_proof() {
        let src = "theorem bad: A |- B\nproof\n  assume h: A\n  exact h\nqed";
        assert!(matches!(verify_source(src), Err(AlifError::Check(_))));
    }

    #[test]
    fn source_with_parse_error() {
        assert!(matches!(verify_source("!!!"), Err(AlifError::Parse(_))));
    }

    #[test]
    fn errors_carry_line_and_column() {
        let src = "axiom a: A\n\ntheorem bad: A |- B\nproof\n  assume h: A\n  exact h\nqed";
        match verify_source(src) {
            Err(AlifError::Check(e)) => {
                let loc = e.location.unwrap();
                assert_eq!((loc.line, loc.column), (6, 3));
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn stdlib_theorems_are_available() {
        let src = "theorem t: X AND Y |- Y AND X\nproof\n  assume h: X AND Y\n  have r: Y AND X := and_comm(h)\n  exact r\nqed";
        verify_source(src).unwrap();
    }

    #[test]
    fn duplicate_theorem_name_is_rejected() {
        let src = format!("{}{}", IDENTITY, IDENTITY);
        match verify_source(&src) {
            Err(AlifError::Load(e)) => assert!(e.message.contains("already defined")),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn stdlib_name_can_be_shadowed_once() {
        verify_source("axiom identity: A").unwrap();
        match verify_source("axiom identity: A\naxiom identity: B") {
            Err(AlifError::Load(e)) => assert!(e.message.contains("already defined")),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn shadowing_replaces_the_stdlib_entry() {
        let src = "axiom identity: P(a)\ntheorem t: |- P(a)\nproof\n  exact identity\nqed";
        verify_source(src).unwrap();
    }

    #[test]
    fn shadowing_a_theorem_with_a_theorem() {
        let src = "theorem and_comm: A |- A\nproof\n  assume h: A\n  exact h\nqed\n\
                   theorem t: X |- X\nproof\n  assume h: X\n  have r: X := and_comm(h)\n  exact r\nqed";
        verify_source(src).unwrap();
    }

    #[test]
    fn rule_name_cannot_be_declared() {
        match verify_source("axiom AndIntro: A") {
            Err(AlifError::Load(e)) => assert!(e.message.contains("built-in rule")),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn theorem_may_use_earlier_axiom() {
        let src = "axiom ax: P(a)\ntheorem t: |- P(a)\nproof\n  exact ax\nqed";
        verify_source(src).unwrap();
    }

    #[test]
    fn theorem_may_not_use_later_axiom() {
        let src = "theorem t: |- P(a)\nproof\n  exact ax\nqed\naxiom ax: P(a)";
        assert!(matches!(verify_source(src), Err(AlifError::Check(_))));
    }

    #[test]
    fn import_from_source_string_is_rejected() {
        match verify_source("import \"x.alif\"") {
            Err(AlifError::Load(e)) => assert!(e.message.contains("file location")),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn file_import_makes_theorems_available() {
        let dir = scratch_dir("import");
        fs::write(dir.join("lib.alif"), IDENTITY).unwrap();
        fs::write(
            dir.join("main.alif"),
            "import \"lib.alif\"\ntheorem t: X |- X\nproof\n  assume h: X\n  have r: X := id_thm(h)\n  exact r\nqed",
        )
        .unwrap();
        verify_file(&dir.join("main.alif")).unwrap();
    }

    #[test]
    fn file_import_in_subdirectory_is_relative_to_importer() {
        let dir = scratch_dir("nested");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("leaf.alif"), IDENTITY).unwrap();
        fs::write(dir.join("sub").join("mid.alif"), "import \"leaf.alif\"").unwrap();
        fs::write(dir.join("main.alif"), "import \"sub/mid.alif\"").unwrap();
        verify_file(&dir.join("main.alif")).unwrap();
    }

    #[test]
    fn diamond_import_loads_once() {
        let dir = scratch_dir("diamond");
        fs::write(dir.join("base.alif"), IDENTITY).unwrap();
        fs::write(dir.join("a.alif"), "import \"base.alif\"").unwrap();
        fs::write(dir.join("b.alif"), "import \"base.alif\"").unwrap();
        fs::write(dir.join("main.alif"), "import \"a.alif\"\nimport \"b.alif\"").unwrap();
        verify_file(&dir.join("main.alif")).unwrap();
    }

    #[test]
    fn import_cycle_is_rejected() {
        let dir = scratch_dir("cycle");
        fs::write(dir.join("a.alif"), "import \"b.alif\"").unwrap();
        fs::write(dir.join("b.alif"), "import \"a.alif\"").unwrap();
        match verify_file(&dir.join("a.alif")) {
            Err(AlifError::Load(e)) => assert!(e.message.contains("cycle")),
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn missing_import_is_located_at_import_site() {
        let dir = scratch_dir("missing");
        fs::write(dir.join("main.alif"), "\nimport \"nope.alif\"").unwrap();
        match verify_file(&dir.join("main.alif")) {
            Err(AlifError::Load(e)) => {
                assert!(e.message.contains("cannot read"));
                assert_eq!(e.location.unwrap().line, 2);
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn error_in_imported_file_names_that_file() {
        let dir = scratch_dir("badimport");
        fs::write(
            dir.join("lib.alif"),
            "theorem bad: A |- B\nproof\n  assume h: A\n  exact h\nqed",
        )
        .unwrap();
        fs::write(dir.join("main.alif"), "import \"lib.alif\"").unwrap();
        match verify_file(&dir.join("main.alif")) {
            Err(AlifError::Check(e)) => {
                let file = e.location.unwrap().file.unwrap();
                assert!(file.ends_with("lib.alif"));
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn missing_file_is_load_error() {
        let path = std::env::temp_dir().join("alif-definitely-missing.alif");
        assert!(matches!(verify_file(&path), Err(AlifError::Load(_))));
    }
}
