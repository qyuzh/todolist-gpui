use bon::bon;
use gpui::*;
use gpui_component::{
    Icon, IconName, Root, StyledExt, Theme,
    button::{Button, ButtonVariants},
    input::{self, InputState, TextInput},
    theme,
};

use std::borrow::Cow;

use anyhow::{Result, anyhow};

use gpui::AssetSource;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "./assets"]
#[include = "icons/**/*"]
#[exclude = "*.DS_Store"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow!("could not find asset at path \"{}\"", path))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect())
    }
}

pub struct StateCreator<'a, 'b, T: 'static> {
    window: &'a mut Window,
    cx: &'a mut Context<'b, T>,
}

impl<'a, 'b, T: 'static> StateCreator<'a, 'b, T> {
    pub fn new(window: &'a mut Window, cx: &'a mut Context<'b, T>) -> Self {
        Self { window, cx }
    }

    pub fn input(&mut self) -> Entity<InputState> {
        self.cx
            .new(|cx| InputState::new(self.window, cx).placeholder("Type..."))
    }
}

struct TodoList {
    input: Entity<InputState>,
    list: Entity<Vec<String>>,
    title: &'static str,
    _sub_handle: Option<Subscription>,
}

#[bon]
impl TodoList {
    #[builder]
    fn new<'b>(window: &mut Window, cx: &mut Context<'b, Self>, title: &'static str) -> Self {
        let input = StateCreator::new(window, cx).input();
        let list = cx.new(|_| Vec::<String>::new());
        let sub_handle = Self::sub_enter(&input, window, cx);
        Self {
            input,
            list,
            title,
            _sub_handle: sub_handle,
        }
    }

    fn sub_enter(
        input: &Entity<InputState>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Subscription> {
        Some(cx.subscribe_in(
            input,
            window,
            |this, _, e: &gpui_component::input::InputEvent, w, c| {
                if let gpui_component::input::InputEvent::PressEnter { .. } = e {
                    this.list.update(c, |list, c| {
                        list.push(this.input.read(c).value().to_string());
                        this.input.update(c, |state, c| state.set_value("", w, c));
                    });
                }
            },
        ))
    }

    fn on_add_handler(&self) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let input = self.input.clone();
        let list1 = self.list.clone();
        move |_, w, c| {
            list1.update(c, |list, c| {
                list.push(input.read(c).value().to_string());
                input.update(c, |state, c| state.set_value("", w, c));
            });
        }
    }

    fn on_remove_handler(
        &self,
        item: String,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let list2 = self.list.clone();
        move |_, _, c| {
            list2.update(c, |list, _c| {
                list.retain(|i| i != &item);
            });
        }
    }
}

impl Render for TodoList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .paddings(20.0)
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .child(format!("{} - GPUI", self.title))
            .child(
                TextInput::new(&self.input).w(px(300.)).suffix(
                    Button::new("Add")
                        .compact()
                        .icon(IconName::Plus)
                        .ghost()
                        .on_click(self.on_add_handler()),
                ),
            )
            .children(self.list.read(cx).iter().enumerate().map(|(index, item)| {
                div()
                    .h_flex()
                    .content_evenly()
                    .w(px(300.))
                    .margins(5.0)
                    .child(format!("T-{index}: {item}"))
                    .child(
                        Button::new(("list", index))
                            .text_color(cx.global::<Theme>().red)
                            .icon(Icon::new(IconName::Minus).size_6())
                            .on_click(self.on_remove_handler(item.clone()))
                            .ghost(),
                    )
            }))
    }
}

fn main() {
    Application::new().with_assets(Assets).run(|cx: &mut App| {
        theme::init(cx);
        input::init(cx);

        cx.open_window(WindowOptions::default(), |window, cx| {
            let root = cx.new(|cx| {
                TodoList::builder()
                    .cx(cx)
                    .window(window)
                    .title("TodoList")
                    .build()
            });
            cx.new(|cx| Root::new(root.into(), window, cx))
        })
        .expect("Failed to open window");
    });
}
