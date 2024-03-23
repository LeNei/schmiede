use askama_axum::{IntoResponse, Template};
use std::ops::Deref;

use crate::common::traits::Displayable;

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login {}

#[derive(Clone)]
pub struct SidebarItem<'a> {
    url: &'a str,
    name: &'a str,
    // Since icons are directly embedded be carefull with xss
    icon: &'a str,
}

const SIDEBAR_ITEM_COUNT: usize = 2;
pub const SIDEBAR_ITEMS: [SidebarItem; SIDEBAR_ITEM_COUNT] = [
    SidebarItem {
        url: "/",
        name: "Home",
        icon: r#"
          <svg
            fill="none"
            viewBox="1 0 24 24"
            stroke-width="2.5"
            stroke="currentColor"
            aria-hidden="true"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M3.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25"
            />
          </svg>"#,
    },
    SidebarItem {
        url: "/users",
        name: "Users",
        icon: r#"
<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z" />
</svg>
        "#,
    },
];

#[derive(Template, Clone)]
#[template(path = "sidebar_layout.html")]
pub struct SidebarLayout {}

#[derive(Template)]
#[template(path = "components/table.html")]
struct Table<'a> {
    title: &'a str,
    description: &'a str,
    add_title: &'a str,
    url: &'a str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Template)]
#[template(path = "pages/table.html")]
struct TablePage<'a> {
    _parent: SidebarLayout,
    title: &'a str,
    description: &'a str,
    add_title: &'a str,
    url: &'a str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Deref for TablePage<'_> {
    type Target = SidebarLayout;

    fn deref(&self) -> &Self::Target {
        &self._parent
    }
}

pub struct TablePageBuilder<'a> {
    title: &'a str,
    description: &'a str,
    add_title: &'a str,
    url: &'a str,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    use_sidebar: bool,
}

impl<'a> TablePageBuilder<'a> {
    pub fn new() -> Self {
        TablePageBuilder {
            title: "",
            description: "",
            add_title: "",
            url: "",
            headers: vec![],
            rows: vec![],
            use_sidebar: true,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }

    pub fn add_title(mut self, add_title: &'a str) -> Self {
        self.add_title = add_title;
        self
    }

    pub fn url(mut self, url: &'a str) -> Self {
        self.url = url;
        self
    }

    pub fn headers<T: Displayable>(mut self) -> Self {
        self.headers = T::table_titles();
        self
    }

    pub fn rows<T: Displayable>(mut self, rows: &Vec<T>) -> Self
where {
        self.rows = rows.iter().map(|r| r.to_table_row()).collect();
        self
    }

    pub fn use_sidebar(mut self, use_sidebar: bool) -> Self {
        self.use_sidebar = use_sidebar;
        self
    }

    pub fn build(self) -> impl IntoResponse {
        if self.use_sidebar {
            TablePage {
                title: self.title,
                description: self.description,
                add_title: self.add_title,
                url: self.url,
                headers: self.headers,
                rows: self.rows,
                _parent: SidebarLayout {},
            }
            .into_response()
        } else {
            Table {
                title: self.title,
                description: self.description,
                add_title: self.add_title,
                url: self.url,
                headers: self.headers,
                rows: self.rows,
            }
            .into_response()
        }
    }
}

impl<'a> From<&'a Table<'a>> for TablePage<'a> {
    fn from(table: &'a Table<'a>) -> Self {
        TablePage {
            title: table.title,
            description: table.description,
            add_title: table.add_title,
            url: table.url,
            headers: table.headers.clone(),
            rows: table.rows.clone(),
            _parent: SidebarLayout {},
        }
    }
}

#[derive(Clone, Debug)]
pub enum FormInputType {
    Text,
    TextArea,
    Number,
    Date,
    Select(Vec<(String, String)>),
}

#[derive(Clone)]
pub struct FormInput<'a> {
    pub label: &'a str,
    pub name: &'a str,
    pub value: Option<String>,
    pub required: bool,
    pub input_type: FormInputType,
}

#[derive(Template)]
#[template(path = "components/upsert_form.html")]
pub struct Form<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub inputs: Vec<FormInput<'a>>,
}

#[derive(Template)]
#[template(path = "pages/form.html")]
pub struct FormPage<'a> {
    pub _parent: SidebarLayout,
    pub title: &'a str,
    pub description: &'a str,
    pub inputs: Vec<FormInput<'a>>,
}

impl Deref for FormPage<'_> {
    type Target = SidebarLayout;

    fn deref(&self) -> &Self::Target {
        &self._parent
    }
}

impl<'a> From<&'a Form<'a>> for FormPage<'a> {
    fn from(form: &'a Form<'a>) -> Self {
        FormPage {
            title: form.title,
            description: form.description,
            inputs: form.inputs.clone(),
            _parent: SidebarLayout {},
        }
    }
}
