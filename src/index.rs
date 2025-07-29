use aaska::fs::PageList;

pub fn index_html(meta: crate::SiteMetadata, post_list: &PageList) -> String {
    let page_links = post_list
        .sorted_by_date()
        .iter()
        .map(|file| {
            let title = file
                .maybe_frontmatter
                .as_ref()
                .and_then(|fm| fm.title.clone())
                .unwrap_or("untitled".to_string());

            let date = file
                .maybe_frontmatter
                .as_ref()
                .and_then(|fm| fm.date.map(|d| d.to_string()))
                .unwrap_or("unknown date".to_string());

            format!(
                "<li><a href=\"/posts/{}\">{}</a> - <em>{}</em></li>",
                file.path.file_name().unwrap().to_str().unwrap(),
                title,
                date
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    maud::html! {
        html {
            head {
                title { "Aaska" }
                link rel="stylesheet" href="/static/style.css" {}
            }
            body {
                h1 { "Welcome to Aaska!" }
                p { "This is the index page." }
                p { "Author: " (meta.author) }

                div {
                    h2 { "Recent Posts" }
                    ul {
                        @if page_links.is_empty() {
                            li { "No posts available." }
                        } @else {
                            (maud::PreEscaped(page_links))
                        }
                    }
                }


            }
        }
    }
    .0
}
