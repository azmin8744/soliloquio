use entities::Post;

pub trait PostUpdateService {
    fn execute(&self, id: i32, title: String, body: String, published: bool) -> Result<Post, String>;
}

pub struct UpdatePost {
    pub post_repository: Box<dyn PostRepository>,
}

impl UpdatePost {
    pub fn new(post_repository: Box<dyn PostRepository>) -> Self {
        Self {
            post_repository,
        }
    }
}

impl PostUpdateService for UpdatePost {
    fn execute(&self, id: i32, title: String, body: String, published: bool) -> Result<Post, String> {
        let post = Post {
            id,
            title,
            body,
            published,
        };

        self.post_repository.update(post)
    }
}

#[cfg(test)]
mod test;

