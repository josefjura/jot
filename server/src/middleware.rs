use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};

use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    db::user::read_user_by_id,
    errors::{AuthError, RestError, RestResult},
    jwt::TokenClaims,
    state::AppState,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> RestResult<impl IntoResponse> {
    let token_option = req
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ").map(|s| s.to_string()));

    let token = if let Some(tk) = token_option {
        tk
    } else {
        Err(RestError::Authorization(AuthError::TokenNotFound))?
    };

    let claims = if let Ok(clm) = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    ) {
        clm.claims
    } else {
        Err(RestError::Authorization(AuthError::TokenInvalid))?
    };

    let user_id = &claims.sub;
    let user = read_user_by_id(&state.db, user_id).await;

    match user {
        Ok(Some(user)) => {
            req.extensions_mut().insert(user);
        }
        Ok(None) => Err(RestError::Internal(
            "Cannot find authorized user".to_string(),
        ))?,
        Err(e) => Err(RestError::Internal(format!(
            "Error while searching for user: {e}"
        )))?,
    }

    Ok(next.run(req).await)
}
