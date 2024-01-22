use iced::widget::{image::Handle, Image};

pub struct WebImage {
    image: Handle,
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebImageMessage {
    Error(String),
    ImageLoaded(Option<Vec<u8>>),
}

impl WebImage {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            image: Handle::from_memory(include_bytes!("../../ava.png")),
            client,
        }
    }

    pub fn load_image(&self, url: String) -> iced::Command<WebImageMessage> {
        let client = self.client.clone();
        iced::Command::perform(
            async move {
                let response = client.get(url).send().await.ok()?;
                let bytes = response.bytes().await.ok()?;
                Some(bytes)
            },
            |bytes| WebImageMessage::ImageLoaded(bytes.map(Vec::from)),
        )
    }

    pub fn update(&mut self, message: WebImageMessage) -> iced::Command<WebImageMessage> {
        match message {
            WebImageMessage::ImageLoaded(bytes) => {
                if let Some(bytes) = bytes {
                    self.image = Handle::from_memory(bytes);
                }
                iced::Command::none()
            }
            WebImageMessage::Error(_) => unreachable!(),
        }
    }

    pub fn view(&self) -> Image<Handle> {
        Image::new(self.image.clone())
    }
}
