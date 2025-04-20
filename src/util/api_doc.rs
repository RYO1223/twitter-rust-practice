use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, Http, HttpAuthScheme}};

use crate::{
    controllers::auth_controller,
    controllers::post_controller,
    models::{post, user}
};

// Define security scheme modifier for OpenAPI docs
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // Add security scheme component
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "bearer_auth", 
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer))
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        post_controller::get_all_posts,
        post_controller::get_post_by_id,
        post_controller::create_post,
        post_controller::update_post,
        post_controller::delete_post,
        auth_controller::register,
        auth_controller::login
    ),
    components(schemas(
        post::Post, 
        post::CreatePostRequest, 
        user::User,
        auth_controller::LoginRequest,
        auth_controller::RegisterRequest,
        auth_controller::AuthResponse
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
