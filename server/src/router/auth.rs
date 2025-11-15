use aide::{
    axum::{
        routing::{delete_with, get_with, post_with},
        ApiRouter, IntoApiResponse,			
    },
    transform::TransformOperation,
};
use axum::{
    extract::{Path, State, Json}, http::{header::SET_COOKIE, StatusCode}, response::{AppendHeaders, IntoResponse}, routing::get, Form
};

use axum_extra::response::Html;
use tower_sessions::cookie::{time::Duration, Cookie, SameSite};
use tracing::error;

use crate::{
    db::auth,
    errors::RestError,
    jwt::create_token,
    model::{
        auth::{ChallengeResult, DeviceCodeRequest, DeviceStatusResponse, LoginRequest, LoginResponse}, LoginUserSchema
    },
    state::AppState,
};

use super::with_auth_middleware;

pub fn auth_routes(app_state: AppState) -> ApiRouter<AppState> {
    ApiRouter::new()
        .merge(auth_routes_public())
        .merge(auth_routes_private(app_state))
}

pub fn auth_routes_public() -> ApiRouter<AppState> {
	ApiRouter::new()
			.api_route("/auth/login", post_with(login_post, login_post_docs))
			.api_route("/auth/logout", post_with(logout_post, logout_post_docs))
			.api_route("/auth/device", post_with(device_post, device_post_docs))
			.api_route(
				"/auth/status/:code",
				get_with(device_status_get, device_status_get_docs),
			)
			// Regular route() for HTML endpoints
			.route("/auth/page/:code", get(device_auth_get).post(device_auth_post))
}


pub fn auth_routes_private(app_state: AppState) -> ApiRouter<AppState> {
	let routes =ApiRouter::new()		
			.api_route(
				"/auth/device/:code",
				delete_with(device_delete, device_delete_docs),
			);
			

			with_auth_middleware(routes, app_state)
}

pub async fn login_post(
    State(state): State<AppState>,
    Json(form_data): Json<LoginUserSchema>,
) -> impl IntoApiResponse {
    let result = auth::check_email_password(
        &form_data.username,
        form_data.password.clone(),
        &state.db,
    )
    .await;

    if form_data.username.is_empty() || form_data.password.is_empty() {
        return RestError::InvalidInput("Username and password are required".to_string())
            .into_response();
    }

		result		
		.and_then(|user| 
			create_token(user.id, state.jwt_secret.as_ref())
			)
		.map(|token|Json(LoginResponse { token }))
		.map_err(RestError::Authorization)		
		.into_response()
}

pub fn login_post_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Authenticate user")
        .description("Authenticate user and receive session token")
        .tag("Authentication")
        .response::<200, ()>()
        .response_with::<400, (), _>(|res| {
            res.description("Invalid request - missing username or password")
        })
        .response_with::<401, (), _>(|res| {
            res.description("Authentication failed - invalid credentials")
        })
}

pub async fn logout_post() -> impl IntoApiResponse {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let headers = AppendHeaders([(SET_COOKIE, cookie.to_string())]);

    (headers, StatusCode::OK)
}

pub fn logout_post_docs(op: TransformOperation) -> TransformOperation {
    op.summary("Logout user")
        .description("Logout user and reject session token")
        .tag("Authentication")
        .response::<200, ()>()
}

pub async fn device_post(
    State(state): State<AppState>,
    Json(req): Json<DeviceCodeRequest>,
) -> impl IntoApiResponse {
    let result = auth::create_device_challenge(req.device_code, &state.db).await;

    if let Err(err) = result {
        error!("{}", err);
        return RestError::Database(err).into_response();
    }

    StatusCode::CREATED.into_response()
}

pub fn device_post_docs(op: TransformOperation) -> TransformOperation {
	op.summary("Create device authorization challenge")
			.description("Creates a device authorization challenge using the provided device code")
			.tag("Device Authorization")					
			.response_with::<201, (), _>(|res| 
					res.description("Device challenge created successfully")
			)
			.response_with::<500, (), _>(|res| 
					res.description("Database error occurred while creating device challenge")
			)
}

pub async fn device_delete(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> impl IntoApiResponse {
    let result = auth::delete_device_challenge(code, &state.db).await;

		match result {
			Ok(true) => StatusCode::NO_CONTENT.into_response(),
			Ok(false) => StatusCode::NOT_FOUND.into_response(),
			Err(err) => {
				error!("{}", err);
				RestError::Database(err).into_response()
			}
		}
}

pub fn device_delete_docs(op: TransformOperation) -> TransformOperation {
	op.summary("Delete device authorization challenge")
			.description("Removes a device authorization challenge using the provided code")
			.tag("Device Authorization")
			.response_with::<204, (), _>(|res| 
					res.description("Device challenge successfully deleted")
			)
			.response_with::<404, (), _>(|res| 
					res.description("No device challenge found for the provided code")
			)
			.response_with::<500, (), _>(|res| 
					res.description("Database error occurred while deleting device challenge")
			)
}

pub async fn device_status_get(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> impl IntoApiResponse {
    let result = auth::get_token_from_device_challenge(code, &state.db).await;

    let result = match result {
				Ok(token) => token,
				Err(err) => {
						error!("{}", err);
						return RestError::Database(err).into_response();
				}
		};

    match result {
        ChallengeResult::Success(access_token) => Json(DeviceStatusResponse { access_token }).into_response(),
        ChallengeResult::NoChallenge => StatusCode::NOT_FOUND.into_response(),
        ChallengeResult::Pending => StatusCode::ACCEPTED.into_response(),
    }
}

pub fn device_status_get_docs(op: TransformOperation) -> TransformOperation {
	op.summary("Check device authorization status")
			.description("Retrieves the current status of a device authorization challenge and returns the authentication token if available")
			.tag("Device Authorization")
			.response_with::<200, Json<LoginResponse>, _>(|res| {
					res.description("Device is authorized")
							.example(LoginResponse {
									token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...".to_string(),
							})
			})
			.response_with::<202, (), _>(|res| 
					res.description("Device authorization is pending")
			)
			.response_with::<404, (), _>(|res| 
					res.description("No device challenge found for the provided code")
			)
			.response_with::<500, (), _>(|res| 
					res.description("Database error occurred while checking device status")
			)
}

pub async fn device_auth_get(
	Path(code): Path<String>,
) -> impl IntoResponse {
	let html = include_str!("../../static/device_auth.html");
	Html(html.replace("<<CODE>>", &code))
}

pub async fn device_auth_post(
	State(state): State<AppState>,
	Path(code): Path<String>,
	Form(data): Form<LoginRequest>,
) -> impl IntoResponse {

	let result = auth::check_email_password(&data.email, data.password, &state.db).await;
	let handle_auth_error = |err: &str| generate_error_page(&code, err, &data.email);

	match result {
		Ok(user) => {
			
			let token = match create_token(user.id, state.jwt_secret.as_ref()) {
				Ok(token) => token,
				Err(err) => {
					error!("{}", err);
					let html = handle_auth_error("Failed to create token");
					return Html(html);
				}
			};

			let done= auth::add_token_to_device_challenge(&code, token, &state.db)
					.await;// TODO: return error page if failed

			match done {
				Ok(true) => {
					let html = include_str!("../../static/device_auth_success.html");
					Html(html.to_string())
				}
				Ok(false) => {
					let error_message = format!("Device code '{}' is not valid", &code);
					let html = handle_auth_error(&error_message);
					Html(html)
				}
				Err(err) => {
					error!("{}", err);
					let html = handle_auth_error("Database error occurred");
					Html(html)
				}
			}
		}
		Err(_) => {
			let html = handle_auth_error("Invalid username or password");
			Html(html)
		}
	}	


}

fn generate_error_page(code: &str, error: &str, email: &str) -> String {
	let html = include_str!("../../static/device_auth_error.html");
	html.replace("<<CODE>>", code)
			.replace("<<ERROR>>", error)
			.replace("<<EMAIL>>", email)
}