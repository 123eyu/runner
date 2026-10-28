use crate::surfaces::remote_client::RemoteItem;
use crate::*;

impl NativeRoot {
    pub(crate) fn open_remote_library(
        &mut self,
        item: RemoteItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.front_page_client(cx).supports(&["catalog"]) {
            return;
        }
        self.dismiss_sidebar_transients(cx);
        self.remote_client.invalidate_library();
        self.select_remote_item(item, cx);
        window.focus(&self.root_focus, cx);
    }
}
