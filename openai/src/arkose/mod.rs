pub mod crypto;
pub mod funcaptcha;
pub mod har;
pub mod murmur;

use std::path::Path;
use std::sync::Once;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use rand::Rng;
use reqwest::header;
use reqwest::impersonate::Impersonate;
use reqwest::redirect::Policy;
use reqwest::Method;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use crate::arkose::crypto::encrypt;
use crate::context::Context;
use crate::debug;

const HEADER: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

static mut CLIENT: Option<reqwest::Client> = None;

static CLIENT_HOLDER: ClientHolder = ClientHolder(Once::new());

pub struct ClientHolder(Once);

impl ClientHolder {
    pub fn get_instance(&self) -> reqwest::Client {
        // Use Once to guarantee initialization only once
        self.0.call_once(|| {
            let client = reqwest::Client::builder()
                .user_agent(HEADER)
                .impersonate(Impersonate::Chrome114)
                .redirect(Policy::none())
                .cookie_store(true)
                .build()
                .unwrap();
            unsafe { CLIENT = Some(client) };
        });
        unsafe {
            CLIENT
                .clone()
                .expect("The requesting client is not initialized")
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum GPT4Model {
    Gpt4model,
    Gpt4browsingModel,
    Gpt4pluginsModel,
    Gpt4Other,
}

impl TryFrom<&str> for GPT4Model {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "gpt-4" => Ok(GPT4Model::Gpt4model),
            "gpt-4-browsing" => Ok(GPT4Model::Gpt4browsingModel),
            "gpt-4-plugins" => Ok(GPT4Model::Gpt4pluginsModel),
            _ => {
                if value.starts_with("gpt-4") || value.starts_with("gpt4") {
                    return Ok(GPT4Model::Gpt4Other);
                }
                Err(())
            }
        }
    }
}

/// curl 'https://tcr9i.chat.openai.com/fc/gt2/public_key/35536E1E-65B4-4D96-9D97-6ADB7EFF8147' --data-raw 'public_key=35536E1E-65B4-4D96-9D97-6ADB7EFF8147'
#[derive(Deserialize, Debug)]
pub struct ArkoseToken {
    token: String,
}

impl ArkoseToken {
    pub fn value(&self) -> &str {
        &self.token
    }

    pub fn valid(&self) -> bool {
        self.token.contains("sup=1|rid=")
    }

    pub async fn new_from_context() -> anyhow::Result<Self> {
        get_arkose_token_from_context().await
    }

    pub async fn new_from_endpoint(endpoint: &str) -> anyhow::Result<Self> {
        get_arkose_token_from_endpoint(endpoint).await
    }

    pub async fn new() -> anyhow::Result<Self> {
        get_arkose_token().await
    }

    pub async fn new_form_har<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        get_arkose_token_from_har(path).await
    }
}

impl From<String> for ArkoseToken {
    fn from(value: String) -> Self {
        ArkoseToken { token: value }
    }
}

impl Serialize for ArkoseToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

async fn get_arkose_token() -> anyhow::Result<ArkoseToken> {
    let bx = serde_json::json!([{"key":"api_type","value":"js"},{"key":"p","value":1},{"key":"f","value":"cdb0697262ceb9aa4e938ae5d9697efd"},{"key":"n","value":"MTY4OTQ0MjY0Mw=="},{"key":"wh","value":"15fb0b896f4534b3970269c17e188322|5ab5738955e0611421b686bc95655ad0"},{"key":"enhanced_fp","value":[{"key":"webgl_extensions","value":"ANGLE_instanced_arrays;EXT_blend_minmax;EXT_color_buffer_half_float;EXT_float_blend;EXT_frag_depth;EXT_shader_texture_lod;EXT_sRGB;EXT_texture_compression_bptc;EXT_texture_compression_rgtc;EXT_texture_filter_anisotropic;OES_element_index_uint;OES_fbo_render_mipmap;OES_standard_derivatives;OES_texture_float;OES_texture_float_linear;OES_texture_half_float;OES_texture_half_float_linear;OES_vertex_array_object;WEBGL_color_buffer_float;WEBGL_compressed_texture_etc;WEBGL_compressed_texture_s3tc;WEBGL_compressed_texture_s3tc_srgb;WEBGL_debug_renderer_info;WEBGL_debug_shaders;WEBGL_depth_texture;WEBGL_draw_buffers;WEBGL_lose_context"},{"key":"webgl_extensions_hash","value":"ccc5c4979d89351fef1dcc0582cdb3d2"},{"key":"webgl_renderer","value":"NVIDIA GeForce GTX 980/PCIe/SSE2"},{"key":"webgl_vendor","value":"Mozilla"},{"key":"webgl_version","value":"WebGL 1.0"},{"key":"webgl_shading_language_version","value":"WebGL GLSL ES 1.0"},{"key":"webgl_aliased_line_width_range","value":"[1, 10]"},{"key":"webgl_aliased_point_size_range","value":"[1, 2047]"},{"key":"webgl_antialiasing","value":"yes"},{"key":"webgl_bits","value":"8,8,24,8,8,0"},{"key":"webgl_max_params","value":"16,192,32768,1024,32768,32,32768,32,16,32,1024"},{"key":"webgl_max_viewport_dims","value":"[32768, 32768]"},{"key":"webgl_unmasked_vendor","value":"NVIDIA Corporation"},{"key":"webgl_unmasked_renderer","value":"NVIDIA GeForce GTX 980/PCIe/SSE2"},{"key":"webgl_vsf_params","value":"23,127,127,23,127,127,23,127,127"},{"key":"webgl_vsi_params","value":"0,24,24,0,24,24,0,24,24"},{"key":"webgl_fsf_params","value":"23,127,127,23,127,127,23,127,127"},{"key":"webgl_fsi_params","value":"0,24,24,0,24,24,0,24,24"},{"key":"webgl_hash_webgl","value":"5890c452638eafb176df4e27cce6e5a3"},{"key":"user_agent_data_brands","value":null},{"key":"user_agent_data_mobile","value":null},{"key":"navigator_connection_downlink","value":null},{"key":"navigator_connection_downlink_max","value":null},{"key":"network_info_rtt","value":null},{"key":"network_info_save_data","value":null},{"key":"network_info_rtt_type","value":null},{"key":"screen_pixel_depth","value":24},{"key":"navigator_device_memory","value":null},{"key":"navigator_languages","value":"en-US,en"},{"key":"window_inner_width","value":0},{"key":"window_inner_height","value":0},{"key":"window_outer_width","value":631},{"key":"window_outer_height","value":1039},{"key":"browser_detection_firefox","value":true},{"key":"browser_detection_brave","value":false},{"key":"audio_codecs","value":"{\"ogg\":\"probably\",\"mp3\":\"maybe\",\"wav\":\"probably\",\"m4a\":\"maybe\",\"aac\":\"maybe\"}"},{"key":"video_codecs","value":"{\"ogg\":\"probably\",\"h264\":\"probably\",\"webm\":\"probably\",\"mpeg4v\":\"\",\"mpeg4a\":\"\",\"theora\":\"\"}"},{"key":"media_query_dark_mode","value":true},{"key":"headless_browser_phantom","value":false},{"key":"headless_browser_selenium","value":true},{"key":"headless_browser_nightmare_js","value":false},{"key":"document__referrer","value":"http://127.0.0.1:8000/"},{"key":"window__ancestor_origins","value":null},{"key":"window__tree_index","value":[0]},{"key":"window__tree_structure","value":"[[]]"},{"key":"window__location_href","value":"https://tcr9i.chat.openai.com/v2/1.5.2/enforcement.64b3a4e29686f93d52816249ecbf9857.html#35536E1E-65B4-4D96-9D97-6ADB7EFF8147"},{"key":"client_config__sitedata_location_href","value":"http://127.0.0.1:8000/arkose.html"},{"key":"client_config__surl","value":"https://tcr9i.chat.openai.com"},{"key":"mobile_sdk__is_sdk"},{"key":"client_config__language","value":null},{"key":"audio_fingerprint","value":"35.73833402246237"}]},{"key":"fe","value":["DNT:unspecified","L:en-US","D:24","PR:1","S:1920,1080","AS:1920,1080","TO:-480","SS:true","LS:true","IDB:true","B:false","ODB:false","CPUC:unknown","PK:Linux x86_64","CFP:699685943","FR:false","FOS:false","FB:false","JSF:Arial,Courier New,Times New Roman","P:Chrome PDF Viewer,Chromium PDF Viewer,Microsoft Edge PDF Viewer,PDF Viewer,WebKit built-in PDF","T:0,false,false","H:16","SWF:false"]},{"key":"ife_hash","value":"63562827dd8cdcf172844085452eb5f4"},{"key":"cs","value":1},{"key":"jsbd","value":"{\"HL\":1,\"NCE\":true,\"DT\":\"\",\"NWD\":\"true\",\"DOTO\":1,\"DMTO\":1}"}]);
    let bv = HEADER;

    let bt = SystemTime::now().duration_since(UNIX_EPOCH)?.as_micros() / 1000000;
    let bw = (bt - (bt % 21600)).to_string();

    let bda = encrypt(&bx.to_string(), &format!("{bv}{bw}"))?;
    #[allow(deprecated)]
    let bda_encoded = base64::encode(&bda);

    let form: [(&str, &str); 8] = [
        ("public_key", "35536E1E-65B4-4D96-9D97-6ADB7EFF8147"),
        ("site", "https://chat.openai.com"),
        ("userbrowser", bv),
        ("capi_version", "1.5.2"),
        ("capi_mode", "lightbox"),
        ("style_theme", "default"),
        ("bda", &bda_encoded),
        ("rnd", &(&rand::thread_rng().gen::<f64>().to_string())),
    ];

    let resp = CLIENT_HOLDER.get_instance()
            .post("https://tcr9i.chat.openai.com/fc/gt2/public_key/35536E1E-65B4-4D96-9D97-6ADB7EFF8147")
            .header(header::USER_AGENT, HEADER)
            .header(header::ACCEPT, "*/*")
            .header(header::ACCEPT_ENCODING, "gzip, deflate, br")
            .header(header::ACCEPT_LANGUAGE, "zh-CN,zh-Hans;q=0.9")
            .header(header::ORIGIN, "https://tcr9i.chat.openai.com")
            .header(header::REFERER, "https://tcr9i.chat.openai.com/v2/1.5.4/enforcement.cd12da708fe6cbe6e068918c38de2ad9.html")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "cors")
            .header("Sec-Fetch-Sitet", "same-origin")
            .form(&form)
            .send().await?;

    if resp.status() != reqwest::StatusCode::OK {
        anyhow::bail!(format!("get arkose token status code {}", resp.status()));
    }

    Ok(resp.json::<ArkoseToken>().await?)
}

/// Build it yourself: https://github.com/gngpp/arkose-generator
async fn get_arkose_token_from_endpoint(endpoint: &str) -> anyhow::Result<ArkoseToken> {
    let resp = CLIENT_HOLDER
        .get_instance()
        .get(endpoint)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await?;
    Ok(resp.json::<ArkoseToken>().await?)
}

async fn get_arkose_token_from_har<P: AsRef<Path>>(path: P) -> anyhow::Result<ArkoseToken> {
    let mut entry = har::parse(path)?;

    let bt = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let bw = bt - (bt % 21600);
    let bv = &entry.bv;
    let bx = &entry.bx;

    let bda = crypto::encrypt(bx, &format!("{bv}{bw}"))?;
    let rnd = format!("{}", rand::Rng::gen::<f64>(&mut rand::thread_rng()));

    #[allow(deprecated)]
    entry
        .body
        .push_str(&format!("&bda={}", base64::encode(bda)));
    entry.body.push_str(&format!("&rnd={rnd}"));

    let client = CLIENT_HOLDER.get_instance();

    let method = Method::from_bytes(entry.method.as_bytes())?;

    let mut builder = client.request(method, entry.url);

    builder = builder.body(entry.body);

    for h in entry.headers.into_iter() {
        if h.name.eq_ignore_ascii_case("cookie") {
            let value = format!(
                "{};{}={}",
                h.value,
                generate_random_string(),
                generate_random_string()
            );
            builder = builder.header(h.name, value);
            continue;
        }
        builder = builder.header(h.name, h.value)
    }

    let res = builder.send().await?;
    match res.error_for_status() {
        Ok(resp) => Ok(resp.json::<ArkoseToken>().await?),
        Err(err) => Err(anyhow::anyhow!(err)),
    }
}

async fn get_arkose_token_from_context() -> anyhow::Result<ArkoseToken> {
    let ctx = Context::get_instance();

    if let Some(path) = ctx.arkose_har_file_path() {
        let token =
            get_arkose_token_and_submit_if_invalid(|| get_arkose_token_from_har(&path)).await?;
        return Ok(token);
    }

    if let Some(ref arkose_token_endpoint) = ctx.arkose_token_endpoint() {
        let token = get_arkose_token_and_submit_if_invalid(|| {
            get_arkose_token_from_endpoint(arkose_token_endpoint)
        })
        .await?;
        return Ok(token);
    }

    if let Some(_) = ctx.yescaptcha_client_key() {
        let token = get_arkose_token_and_submit_if_invalid(get_arkose_token).await?;
        return Ok(token);
    }

    anyhow::bail!("There is no way to get arkose token")
}

async fn get_arkose_token_and_submit_if_invalid<F, Fut>(get_token: F) -> anyhow::Result<ArkoseToken>
where
    F: FnOnce() -> Fut,
    Fut: futures_core::Future<Output = anyhow::Result<ArkoseToken>>,
{
    let ctx = Context::get_instance();
    let arkose_token = get_token().await?;
    if arkose_token.valid() {
        Ok(arkose_token)
    } else if let Some(ref key) = ctx.yescaptcha_client_key() {
        submit_captcha(key, arkose_token).await
    } else {
        anyhow::bail!("No yescaptcha_client_key to submit captcha")
    }
}

async fn submit_captcha(key: &str, arkose_token: ArkoseToken) -> anyhow::Result<ArkoseToken> {
    let session = funcaptcha::start_challenge(arkose_token.value())
        .await
        .map_err(|error| anyhow::anyhow!(format!("Error creating session: {}", error)))?;

    let funcaptcha = anyhow::Context::context(session.funcaptcha(), "valid funcaptcha error")?;

    let answer =
        funcaptcha::yescaptcha::submit_task(key, &funcaptcha.image, &funcaptcha.instructions)
            .await?;

    return match session.submit_answer(answer).await {
        Ok(_) => {
            let mut rng = rand::thread_rng();
            let rid = rng.gen_range(1..100);
            Ok(ArkoseToken::from(format!(
                "{}|sup=1|rid={rid}",
                arkose_token.value()
            )))
        }
        Err(err) => {
            debug!("submit funcaptcha answer error: {err}");
            Ok(arkose_token)
        }
    };
}

pub fn generate_random_string() -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    let mut rng = rand::thread_rng();
    let length = rng.gen_range(5..=15);

    let result: String = (0..length)
        .map(|_| {
            let index = rng.gen_range(0..CHARSET.len());
            CHARSET[index] as char
        })
        .collect();

    result
}
