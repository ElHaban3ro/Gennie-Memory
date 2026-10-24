use crate::classifier::DecisionError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Issue {
    pub code: &'static str,
    pub stage: &'static str,
    pub component: &'static str,
    pub message: &'static str,
    pub cause_code: Option<&'static str>,
    pub http_status: Option<u16>,
    pub retryable: Option<bool>,
    pub retry_after_ms: Option<u64>,
    pub suggested_action: &'static str,
}
impl Issue {
    pub fn local(
        code: &'static str,
        stage: &'static str,
        message: &'static str,
        action: &'static str,
    ) -> Self {
        Self {
            code,
            stage,
            component: "gennie-memory",
            message,
            cause_code: None,
            http_status: None,
            retryable: None,
            retry_after_ms: None,
            suggested_action: action,
        }
    }
    pub fn provider(e: &DecisionError) -> Self {
        let (code, message, retryable, action) = match (e.code, e.http_status) {
            ("provider_input_limit", _) => (
                "provider_input_limit",
                "La petición completa supera el límite local de 128 KiB antes de enviarse al proveedor.",
                Some(false),
                "reduce_input",
            ),
            ("deadline_not_supported", _) => (
                "unsupported_deadline",
                "El adaptador no implementa el plazo requerido por el contrato v1.",
                Some(false),
                "choose_supported_adapter",
            ),
            ("missing_api_key", _) => (
                "credential_missing",
                "Hay recuerdos candidatos, pero falta una credencial válida de OpenRouter para seleccionarlos.",
                Some(false),
                "configure_credentials",
            ),
            (_, Some(401 | 403)) => (
                "provider_access_rejected",
                "Hay recuerdos candidatos, pero OpenRouter rechazó la autenticación o el acceso.",
                Some(false),
                "check_access",
            ),
            (_, Some(402)) => (
                "provider_payment_required",
                "Hay recuerdos candidatos, pero OpenRouter rechazó la selección por un requisito de pago o créditos.",
                Some(false),
                "check_billing",
            ),
            (_, Some(429)) => (
                "provider_rate_limited",
                "Hay recuerdos candidatos, pero OpenRouter limitó las peticiones de selección.",
                Some(true),
                "wait_before_retry",
            ),
            ("request_timeout" | "operation_deadline", _) | (_, Some(408 | 504)) => (
                "provider_timeout",
                "Hay recuerdos candidatos, pero la selección agotó el tiempo de espera.",
                Some(true),
                "review_deadline",
            ),
            (_, Some(500..=599)) => (
                "provider_unavailable",
                "Hay recuerdos candidatos, pero OpenRouter o su proveedor no pudieron completar la selección.",
                Some(true),
                "check_provider_status",
            ),
            ("transport_error", _) => (
                "provider_transport_error",
                "Hay recuerdos candidatos, pero falló la comunicación con OpenRouter; la causa concreta no está confirmada.",
                None,
                "check_connection",
            ),
            ("response_too_large", _) => (
                "provider_response_limit",
                "La respuesta de selección superó el límite local de tamaño.",
                Some(false),
                "inspect_provider_response",
            ),
            (
                "invalid_response"
                | "invalid_choice"
                | "invalid_usage"
                | "per_memory_invalid_response"
                | "per_memory_invalid_relation",
                _,
            ) => (
                "provider_invalid_response",
                "La respuesta del proveedor no cumple el contrato de selección.",
                Some(false),
                "inspect_provider_response",
            ),
            (_, Some(400..=499)) => (
                "provider_request_rejected",
                "OpenRouter rechazó la solicitud de selección; no se atribuye una causa más específica sin evidencia.",
                Some(false),
                "inspect_request",
            ),
            _ => (
                "provider_failure_unclassified",
                "No se pudo completar la selección; se conserva el código de causa para diagnóstico.",
                None,
                "inspect_trace",
            ),
        };
        let mut result = Self {
            code,
            stage: "classification",
            component: "openrouter",
            message,
            cause_code: Some(e.code),
            http_status: e.http_status,
            retryable,
            retry_after_ms: None,
            suggested_action: action,
        };
        if let Some(detail) = &e.diagnostic {
            result.retry_after_ms = detail.retry_after_ms;
            let mapped = match detail.code {
                "provider_budget_exceeded" => Some((
                    "OpenRouter rechazó la selección porque se agotó un presupuesto de la cuenta, espacio o clave.",
                    "review_provider_budget",
                    false,
                )),
                "provider_in_flight_budget" => Some((
                    "OpenRouter limitó temporalmente el gasto de peticiones en curso.",
                    "wait_before_retry",
                    true,
                )),
                "provider_context_limit" => Some((
                    "El proveedor rechazó la selección porque la entrada supera su ventana de contexto.",
                    "reduce_input",
                    false,
                )),
                "provider_content_rejected" => Some((
                    "El proveedor rechazó la solicitud por su política de contenido.",
                    "review_request",
                    false,
                )),
                "provider_authentication_rejected" => Some((
                    "OpenRouter rechazó la credencial de selección.",
                    "check_credentials",
                    false,
                )),
                "provider_payment_required" => Some((
                    "Hay recuerdos candidatos, pero OpenRouter rechazó la selección por un requisito de pago o créditos.",
                    "check_billing",
                    false,
                )),
                "provider_rate_limited" => Some((
                    "Hay recuerdos candidatos, pero OpenRouter limitó las peticiones de selección.",
                    "wait_before_retry",
                    true,
                )),
                "provider_timeout" => Some((
                    "Hay recuerdos candidatos, pero el proveedor agotó el tiempo de selección.",
                    "review_deadline",
                    true,
                )),
                "provider_unavailable" => Some((
                    "Hay recuerdos candidatos, pero el proveedor no está disponible para seleccionarlos.",
                    "check_provider_status",
                    true,
                )),
                _ => None,
            };
            if let Some((message, action, retry)) = mapped {
                result.code = detail.code;
                result.message = message;
                result.suggested_action = action;
                result.retryable = Some(retry);
            }
        }
        result
    }
}
