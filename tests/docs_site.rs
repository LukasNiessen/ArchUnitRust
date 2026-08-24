use std::{collections::BTreeMap, fs, path::Path};

use regex::Regex;

const DOCS: &str = "docs";
const PAGES: [(&str, &str, usize); 10] = [
    ("index.md", "ArchUnitRust", 1),
    ("grammar.md", "The grammar", 2),
    ("patterns.md", "Patterns and identifiers", 3),
    ("files.md", "The files family", 4),
    ("layers.md", "The layers family", 5),
    ("slices.md", "The slices family", 6),
    ("metrics.md", "The metrics family", 7),
    ("graph.md", "Dependency-graph reports", 8),
    ("running.md", "Running a rule", 9),
    ("internals.md", "How it works", 10),
];

#[derive(Debug)]
struct Page {
    content: String,
    front_matter: BTreeMap<String, String>,
}

#[test]
fn every_expected_chapter_has_complete_ordered_front_matter() {
    let actual = fs::read_dir(DOCS)
        .expect("docs directory should be readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|value| value == "md"))
        .filter_map(|entry| {
            let content = fs::read_to_string(entry.path()).ok()?;
            content
                .lines()
                .next()
                .is_some_and(|line| line == "---")
                .then(|| entry.file_name().to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    let expected = PAGES
        .iter()
        .map(|(name, _, _)| (*name).to_owned())
        .collect::<Vec<_>>();

    assert_eq!(actual.len(), expected.len());
    for expected_page in &expected {
        assert!(actual.contains(expected_page), "missing {expected_page}");
    }

    for (name, title, order) in PAGES {
        let page = read_page(name);
        assert_eq!(
            page.front_matter.get("layout").map(String::as_str),
            Some("default")
        );
        assert_eq!(
            page.front_matter.get("title").map(String::as_str),
            Some(title)
        );
        assert_eq!(
            page.front_matter
                .get("nav_order")
                .and_then(|value| value.parse::<usize>().ok()),
            Some(order)
        );
        assert!(
            page.front_matter
                .get("description")
                .is_some_and(|description| !description.trim().is_empty()),
            "{name} needs a description"
        );
        let heading = page
            .content
            .lines()
            .find_map(|line| line.strip_prefix("# "));
        assert_eq!(
            heading,
            Some(title),
            "{name} title and first heading differ"
        );
    }
}

#[test]
fn every_local_page_link_and_fragment_resolves() {
    let links = Regex::new(r"\[[^\]]+\]\(([^)]+)\)").expect("link regex should compile");

    for (name, _, _) in PAGES {
        let page = read_page(name);
        let mut local_links = 0;
        for captures in links.captures_iter(&page.content) {
            let target = captures.get(1).expect("capture should exist").as_str();
            if target.starts_with("https://") || target.contains("{{ site.") {
                continue;
            }
            assert!(
                !target.starts_with("http://"),
                "{name} uses insecure link {target}"
            );

            let (file, fragment) = target.split_once('#').unwrap_or((target, ""));
            let target_name = if file.is_empty() { name } else { file };
            assert!(
                target_name.ends_with(".md"),
                "{name} uses non-page link {target}"
            );
            let target_path = Path::new(DOCS).join(target_name);
            assert!(
                target_path.is_file(),
                "{name} links to missing {target_name}"
            );
            local_links += 1;

            if fragment.is_empty() {
                continue;
            }
            let target_content =
                fs::read_to_string(&target_path).expect("linked page should be readable");
            let has_fragment = target_content
                .lines()
                .filter_map(markdown_heading)
                .any(|heading| anchor_for(heading) == fragment);
            assert!(has_fragment, "{name} links to missing fragment {target}");
        }
        assert!(
            local_links > 0,
            "{name} should link to another guide chapter"
        );
    }
}

#[test]
fn landing_page_links_every_other_chapter() {
    let landing = read_page("index.md");
    for (name, _, _) in PAGES.into_iter().skip(1) {
        assert!(
            landing.content.contains(&format!("]({name})")),
            "landing page does not link {name}"
        );
    }
}

#[test]
fn every_chapter_is_attached_to_the_doctest_host() {
    let host = fs::read_to_string("src/site_docs.rs").expect("doctest host should be readable");
    let rust_blocks =
        Regex::new(r"(?m)^```rust(?:,no_run)?\s*$").expect("Rust fence regex should compile");
    let mut examples = 0;

    for (name, _, _) in PAGES {
        assert!(
            host.contains(&format!("include_str!(\"../docs/{name}\")")),
            "{name} is not compiled by src/site_docs.rs"
        );
        examples += rust_blocks.find_iter(&read_page(name).content).count();
    }
    assert!(
        examples >= 25,
        "the guide should contain substantial compiled examples"
    );

    let library = fs::read_to_string("src/lib.rs").expect("library root should be readable");
    assert!(library.contains("#[cfg(doctest)]\nmod site_docs;"));
}

#[test]
fn site_shell_is_accessible_portable_and_excludes_architecture_records() {
    let config = fs::read_to_string("docs/_config.yml").expect("site config should be readable");
    assert!(config.contains("baseurl: /ArchUnitRust"));
    assert!(config.contains("- jekyll-relative-links"));
    assert!(config.contains("- PORTING_PLAN.md"));
    assert!(config.contains("- adr"));

    let layout =
        fs::read_to_string("docs/_layouts/default.html").expect("layout should be readable");
    assert!(layout.contains("class=\"skip\""));
    assert!(layout.contains("<main id=\"content\""));
    assert!(layout.contains("aria-current=\"page\""));
    assert!(layout.contains("property=\"og:image\""));
    assert!(layout.contains("name=\"twitter:card\""));
    assert!(layout.contains("relative_url"));

    let social_preview = fs::metadata("docs/assets/og.png")
        .expect("the social preview image should be present in the site source");
    assert!(social_preview.len() > 100_000);

    let css = fs::read_to_string("docs/assets/docs.css").expect("stylesheet should be readable");
    assert!(css.contains("prefers-color-scheme: dark"));
    assert!(css.contains("prefers-reduced-motion: reduce"));
    assert!(css.contains(":focus-visible"));
}

#[test]
fn pages_workflow_builds_the_guide_and_api_from_the_same_commit() {
    let workflow = fs::read_to_string(".github/workflows/pages.yml")
        .expect("Pages workflow should be readable");
    for required in [
        "branches: [main]",
        "cargo +1.85.0 test --doc --all-features",
        "cargo +1.85.0 test --test docs_site",
        "source: ./docs",
        "cargo +1.85.0 doc --all-features --no-deps",
        "actions/upload-pages-artifact@v5",
        "actions/deploy-pages@v5",
    ] {
        assert!(
            workflow.contains(required),
            "workflow is missing {required}"
        );
    }
}

fn read_page(name: &str) -> Page {
    let content = fs::read_to_string(Path::new(DOCS).join(name))
        .unwrap_or_else(|error| panic!("could not read {name}: {error}"));
    let mut lines = content.lines();
    assert_eq!(lines.next(), Some("---"), "{name} needs front matter");
    let mut front_matter = BTreeMap::new();
    for line in &mut lines {
        if line == "---" {
            break;
        }
        let (key, value) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("invalid front matter in {name}: {line}"));
        front_matter.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    Page {
        content,
        front_matter,
    }
}

fn markdown_heading(line: &str) -> Option<&str> {
    let heading = line.trim_start_matches('#');
    (heading.len() < line.len()).then_some(heading.trim())
}

fn anchor_for(heading: &str) -> String {
    heading
        .chars()
        .filter_map(|character| match character {
            character if character.is_alphanumeric() => Some(character.to_ascii_lowercase()),
            ' ' | '-' => Some('-'),
            _ => None,
        })
        .collect()
}
