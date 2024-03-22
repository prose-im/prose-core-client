// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use mime::Mime;
use std::path::Path;

pub trait PathExt {
    fn media_type(&self) -> Mime;
}

impl PathExt for Path {
    // https://github.com/abonander/mime_guess/issues/88
    fn media_type(&self) -> Mime {
        let media_type = mime_guess::from_path(self).first_or(mime::APPLICATION_OCTET_STREAM);

        if media_type.type_() == mime::AUDIO && media_type.subtype() == "m4a" {
            return "audio/mp4".parse().unwrap();
        }

        media_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_mp4() {
        let path = Path::new("audio-file.m4a");

        let mime_guess_type = mime_guess::from_path(path).first().unwrap();
        assert_eq!(mime_guess_type.type_(), mime::AUDIO);
        assert_eq!(mime_guess_type.subtype(), "m4a");

        let our_type = path.media_type();
        assert_eq!(our_type.type_(), mime::AUDIO);
        assert_eq!(our_type.subtype(), "mp4");
    }
}
