use gpui::{AssetSource, SharedString};
use std::borrow::Cow;

pub(super) struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let body = match path {
            "plus" => "<path d='M12 5v14M5 12h14'/>",
            "arrow" => "<path d='m10 5 7 7-7 7M17 12H4'/>",
            "close" => "<path d='m6 6 12 12M6 18 12 6'/>",
            "check" => "<path d='m5 12 4 4L19 6'/>",
            "chevron" => "<path d='m9 5 7 7-7 7'/>",
            "copy" => {
                "<rect x='8' y='8' width='12' height='13' rx='3'/><path d='M15 4H6a3 3 0 0 0-3 3v9'/>"
            }
            "export" => {
                "<path d='M12 15V3m-4 4 4-4 4 4M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6'/>"
            }
            "folder" => {
                "<path d='M3 8V5a2 2 0 0 1 2-2h5l3 4h6a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8Z'/><path d='M3 8h18'/>"
            }
            "video" => {
                "<rect x='3' y='3' width='18' height='18' rx='4'/><path d='m10 8 6 4-6 4Z'/>"
            }
            "wave" => "<path d='M3 10v4m4-7v10m5-14v18m5-14v10m4-7v4'/>",
            "document" => {
                "<path d='M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9l-6-6Z'/><path d='M14 3v6h6M8 13h8M8 17h5'/>"
            }
            "settings" => {
                "<path d='M4 7h16M4 17h16'/><circle cx='9' cy='7' r='3' fill='#fff' stroke='none'/><circle cx='15' cy='17' r='3' fill='#fff' stroke='none'/>"
            }
            "shield" => {
                "<path d='m12 3 8 3v6c0 5-8 9-8 9s-8-4-8-9V6l8-3Z'/><path d='m8 12 3 3 5-6'/>"
            }
            "stop" => "<rect x='6' y='6' width='12' height='12' rx='2' fill='#fff' stroke='none'/>",
            "spark" => "<path d='m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5L12 3Z'/>",
            "remove" => "<path d='M4 6h16M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7m4-7v7'/>",
            _ => return Ok(None),
        };
        Ok(Some(Cow::Owned(format!("<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='#fff' stroke-width='1.65' stroke-linecap='round' stroke-linejoin='round'>{body}</svg>").into_bytes())))
    }

    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
