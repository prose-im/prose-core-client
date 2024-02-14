// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use tracing::error;

use prose_xmpp::stanza::media_sharing::File;

use crate::domain::messaging::models::Thumbnail;

pub trait FileExt {
    fn best_thumbnail_representation(&self) -> Option<Thumbnail>;
}

impl FileExt for File {
    fn best_thumbnail_representation(&self) -> Option<Thumbnail> {
        self.thumbnails
            .iter()
            .find_map(|t| match Thumbnail::try_from(t.clone()) {
                Ok(t) => Some(t),
                Err(err) => {
                    error!("Encountered invalid thumbnail in file: {}", err.to_string());
                    None
                }
            })
    }
}
