use std::borrow::Cow;
use std::sync::Once;

use vivarium_rs::{Texts, install_debug_mode, install_texts};

static INSTALL: Once = Once::new();

/// 库自带的文案是英文；应用侧一次性注入中文。
///
/// 必须早于任何一次库调用:库的 `texts()` 是 `OnceLock::get_or_init(默认英文)`,
/// 先被调用就会永久冻结英文目录。重复调用是 no-op（测试进程内会多次构造 app）。
pub fn install_chinese_texts() {
    INSTALL.call_once(|| {
        // 内部错误一律不外泄:库会读环境变量 VIVARIUM_DEBUG,为 1/true/on 时在 500 响应里
        // 额外回显源错误文本,破坏本仓「internal 不带 source 文本」的不变量,故显式钉死为关。
        let _ = install_debug_mode(false);

        let _ = install_texts(Texts {
            data_parse: "数据解析失败".into(),
            bad_request: "请求无效".into(),
            validation: "校验失败".into(),
            unauthorized: "未授权".into(),
            forbidden: "权限不足".into(),
            not_found: "未找到".into(),
            conflict: "资源已存在".into(),
            too_many_requests: "请求过于频繁".into(),
            invalid_refresh: "无效或已过期的刷新令牌".into(),
            internal: "内部错误".into(),
            database: "数据库错误".into(),
            // 4xx 的源错误文本直接作为 message(如 "用户 id 无效: abc"),
            // tests/api.rs 断言该文案,故必须打开。
            echo_details: true,
        });
    });
}

/// 库自有 schema 的英文描述 → 中文,返回被替换的条数。
///
/// 库的 `ToSchema` 描述来自 rustdoc(英文);`openapi::localize` 把 `components.schemas` 下
/// 每一条描述交给闭包,`None` 表示保持原样。操作级 summary/description 来自应用侧
/// `#[utoipa::path]`,本就是中文,不在遍历范围内。
///
/// 短描述逐字匹配;两条多段长描述只匹配稳定的首句,避免把库的整段散文抄进本仓。
/// 上游若改写文案,匹配会失效(描述退回英文)——`tests/api.rs` 的中文覆盖断言会失败并提示补表。
pub fn localize_schema(api: &mut utoipa::openapi::OpenApi) -> usize {
    vivarium_rs::openapi::localize(api, |text| {
        let zh = match text {
            "The unified response envelope." => {
                "统一响应信封:code=0 成功,非 0 为 HTTP 状态码;data 恒序列化(错误时为 null)"
            }
            "`0` on success, otherwise the HTTP status code of the failure." => {
                "0 表示成功,否则为失败的 HTTP 状态码"
            }
            "The validation violations; present only for a failed validation." => {
                "校验失败明细,仅在校验失败时出现"
            }
            "A short human-readable message; `\"ok\"` on success." => {
                "简短的可读消息;成功时为 \"ok\""
            }
            "One failed rule for one field." => "某个字段的一条失败规则",
            "The rule that failed (`\"length\"`, `\"email\"`, …)." => {
                "失败的规则名(如 \"length\"、\"email\")"
            }
            "The message the DTO declared for this rule, when it declared one." => {
                "DTO 为该规则声明的消息(声明了才有)"
            }
            t if t.starts_with("The parameters of the failed rule") => {
                "失败规则的参数(如 {\"min\": 3, \"max\": 20})。提交的字段值刻意不回显:validator 会把它记为 \
                 params.value,原样回显会把原始输入(口令、令牌)返回给客户端。"
            }
            t if t.starts_with("Every violation, grouped by field") => {
                "按字段分组的全部违规。键为扁平化的字段路径:顶层字段是 email,嵌套结构体字段是 \
                 profile.email,列表元素是 items[0].name。映射有序,故序列化结果稳定。"
            }
            _ => return None,
        };
        Some(Cow::Borrowed(zh))
    })
}
