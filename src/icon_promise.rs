use std::future::Future;
use futures::task::{Context, Poll};
use futures::future;
use news_flash::NewsFlashError;
use news_flash::models::FavIcon;

#[derive(Clone)]
pub struct IconPromise<F: Future<Output = Result<FavIcon, NewsFlashError>>> {
    future: F,
    icon: Option<FavIcon>,
    resolved: bool,
}

impl<F: Future<Output = Result<FavIcon, NewsFlashError>>> IconPromise<F> {
    pub fn new(future: F) -> Self {
        IconPromise {
            future,
            icon: None,
            resolved: false,
        }
    }

    pub async fn get_icon(&mut self, ctx: &mut Context<'_>) -> Option<FavIcon> {
        self.resolved = true;
        future::poll_fn(|ctx| self.poll_icon(ctx)).await
    }

    fn poll_icon(&mut self, ctx: &mut Context) -> Poll<Option<FavIcon>> {
        if self.resolved {
            //future::ready(self.icon).poll(ctx)
        } else {
            //self.future.poll(ctx)
        }
        Poll::Ready(None)
    }
}