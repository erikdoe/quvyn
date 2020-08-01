use ammonia::clean;
use pulldown_cmark::{Parser, Options, html::push_html};


pub fn md_to_html(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);

    let md_parse = Parser::new_ext(md, options);
    let mut unsafe_html = String::new();
    push_html(&mut unsafe_html, md_parse);

    clean(&*unsafe_html)
}


#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    fn normalize_ws(input: &str) -> String {
        let re = Regex::new(r"\s+").unwrap();
        re.replace_all(input, " ").to_string()
    }

    #[test]
    fn converts_simple_markdown() {
        let md = r#" * list _i_"#;

        let html = md_to_html(md);

        assert_eq!(normalize_ws(&html), "<ul> <li>list <em>i</em></li> </ul> ");
    }

    #[test]
    fn adds_code_blocks_closing_despite_misplaced_end_marker() {
        let md = r#"```
                int foo() { return 3; }
                ```"#;

        let html = md_to_html(md);

        assert_eq!(normalize_ws(&html), r#"<pre><code> int foo() { return 3; } ```</code></pre> "#);
    }

    #[test]
    fn removes_harmful_html() {
        let md = r#"x <iframe src="foo"></iframe> y"#;

        let html = md_to_html(md);

        assert_eq!(normalize_ws(&html), r#"<p>x y</p> "#);
    }

}
