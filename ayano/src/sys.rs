use std::os::raw::c_void;

#[link(name = "httpapi")]
unsafe extern "system" {
    
    // -------------------- server -> api --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpinitialize
    pub fn HttpInitialize(
        version: HttpApiVersion,
        flags: u32,
        p_reserved: *mut c_void,
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpterminate
    pub fn HttpTerminate(
        flags: u32,
        p_reserved: *mut c_void,
    ) -> u32;
    
    // -------------------- server -> session --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreateserversession
    pub fn HttpCreateServerSession(
        version: HttpApiVersion,
        p_server_session_id: *mut u64, // HTTP_OPAQUE_ID
        reserved: u32,
    ) -> u32;
    
    // https://learn.microsoft.com/es-es/windows/win32/api/http/nf-http-httpcloseserversession
    pub fn HttpCloseServerSession(
        server_session_id: u64, // HTTP_OPAQUE_ID
    ) -> u32;
    
    // -------------------- server -> queue --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreaterequestqueue
    pub fn HttpCreateRequestQueue(
        version: HttpApiVersion,
        p_name: *const u16, // PCWSTR
        p_security_attributes: *const c_void, // SECURITY_ATTRIBUTES
        flags: u32,
        p_req_queue_handle: *mut *mut c_void, // HANDLE
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcloserequestqueue
    pub fn HttpCloseRequestQueue(
        h_req_queue_handle: *mut c_void, // HANDLE
    ) -> u32;
    
    // -------------------- server -> url_group --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreateurlgroup
    pub fn HttpCreateUrlGroup(
        server_session_id: u64, // HTTP_OPAQUE_ID
        p_url_group_id: *mut u64, // HTTP_OPAQUE_ID
        reserved: u32,
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpseturlgroupproperty
    pub fn HttpSetUrlGroupProperty(
        url_group_id: u64, // HTTP_OPAQUE_ID
        property: u32,
        p_property_information: *const c_void,
        property_information_length: u32,
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpaddurltourlgroup
    pub fn HttpAddUrlToUrlGroup(
        url_group_id: u64, // HTTP_OPAQUE_ID
        p_fully_qualified_url: *const u16, // PCWSTR
        url_context: u64,
        reserved: u32,
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcloseurlgroup
    pub fn HttpCloseUrlGroup(
        url_group_id: u64, // HTTP_OPAQUE_ID
    ) -> u32;
    
    // -------------------- request --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpreceivehttprequest
    pub fn HttpReceiveHttpRequest(
        req_queue_handle: *mut c_void, // HANDLE
        request_id: u64, // HTTP_OPAQUE_ID
        flags: u32,
        p_request_buffer: *mut c_void, // HTTP_REQUEST
        request_buffer_length: u32,
        p_bytes_received: *mut u32,
        p_overlapped: *mut c_void, // OVERLAPPED
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpreceiverequestentitybody
    pub fn HttpReceiveRequestEntityBody(
        req_queue_handle: *mut c_void, // HANDLE
        request_id: u64, // HTTP_OPAQUE_ID
        flags: u32,
        p_entity_buffer: *mut c_void,
        entity_buffer_length: u32,
        p_bytes_returned: *mut u32,
        p_overlapped: *mut c_void, // OVERLAPPED
    ) -> u32;
    
    // -------------------- response --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpsendhttpresponse
    pub fn HttpSendHttpResponse(
        req_queue_handle: *mut c_void, // HANDLE
        request_id: u64, // HTTP_OPAQUE_ID
        flags: u32,
        p_http_response: *const HttpResponseV2,
        p_cache_policy: *const c_void, // HTTP_CACHE_POLICY
        p_bytes_sent: *mut u32,
        p_reserved1: *const c_void,
        reserved2: u32,
        p_overlapped: *mut c_void, // OVERLAPPED
        p_log_data: *const c_void, // HTTP_LOG_DATA
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpsendresponseentitybody
    pub fn HttpSendResponseEntityBody(
        req_queue_handle: *mut c_void, // HANDLE
        request_id: u64, // HTTP_OPAQUE_ID
        flags: u32,
        entity_chunk_count: u16,
        p_entity_chunks: *const HttpDataChunk,
        p_bytes_sent: *mut u32,
        p_reserved1: *const c_void,
        reserved2: u32,
        p_overlapped: *mut c_void, // OVERLAPPED
        p_log_data: *const c_void, // HTTP_LOG_DATA
    ) -> u32;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcancelhttprequest
    pub fn HttpCancelHttpRequest(
        req_queue_handle: *mut c_void, // HANDLE
        request_id: u64, // HTTP_OPAQUE_ID
        p_overlapped: *mut c_void, // OVERLAPPED
    ) -> u32;
    
}

#[repr(C)]
pub struct HttpApiVersion {
    pub http_api_major_version: u16,
    pub http_api_minor_version: u16,
}

#[repr(C)]
#[derive(PartialEq)]
pub struct HttpVersion {
    pub major_version: u16,
    pub minor_version: u16,
}

#[repr(C)]
pub struct HttpBindingInfo {
    pub flags: u32, // HTTP_PROPERTY_FLAGS
    pub request_queue_handle: *mut c_void, // HANDLE
}

#[repr(C)]
pub struct HttpCookedUrl {
    pub full_url_length: u16,
    pub host_length: u16,
    pub abs_path_length: u16,
    pub query_string_length: u16,
    pub p_full_url: *const u16, // PCWSTR
    pub p_host: *const u16, // PCWSTR
    pub p_abs_path: *const u16, // PCWSTR
    pub p_query_string: *const u16, // PCWSTR
}

#[repr(C)]
pub struct HttpKnownHeader {
    pub raw_value_length: u16,
    pub p_raw_value: *const i8, // PCSTR
}

#[repr(C)]
pub struct HttpRequestHeaders {
    pub unknown_header_count: u16,
    pub p_unknown_headers: *const c_void, // HTTP_UNKNOWN_HEADER
    pub trailer_count: u16,
    pub p_trailers: *const c_void, // HTTP_UNKNOWN_HEADER
    pub known_headers: [HttpKnownHeader; HTTP_HEADER_REQUEST_MAXIMUM],
}

#[repr(C)]
pub struct HttpRequestV2 {
    pub flags: u32,
    pub connection_id: u64,
    pub request_id: u64,
    pub url_context: u64,
    pub version: HttpVersion,
    pub verb: u32,
    pub unknown_verb_length: u16,
    pub raw_url_length: u16,
    pub p_unknown_verb: *const i8, // PCSTR
    pub p_raw_url: *const i8, // PCSTR
    pub cooked_url: HttpCookedUrl,
    pub address: [*const c_void; 2], // HTTP_TRANSPORT_ADDRESS
    pub headers: HttpRequestHeaders,
    pub bytes_received: u64,
    pub entity_chunk_count: u16,
    pub p_entity_chunks: *const c_void,
    pub raw_connection_id: u64,
    pub p_ssl_info: *const c_void,
    pub request_info_count: u16,
    pub p_request_info: *const c_void, // HTTP_REQUEST_INFO
}

#[repr(C)]
pub struct HttpResponseHeaders {
    pub unknown_header_count: u16,
    pub p_unknown_headers: *const c_void, // HTTP_UNKNOWN_HEADER
    pub trailer_count: u16,
    pub p_trailers: *const c_void, // HTTP_UNKNOWN_HEADER
    pub known_headers: [HttpKnownHeader; HTTP_HEADER_RESPONSE_MAXIMUM],
}

#[repr(C)]
pub struct HttpResponseV2 {
    pub flags: u32,
    pub version: HttpVersion,
    pub status_code: u16,
    pub reason_length: u16,
    pub p_reason: *const i8, // PCSTR
    pub headers: HttpResponseHeaders,
    pub entity_chunk_count: u16,
    pub p_entity_chunks: *const HttpDataChunk,
    pub response_info_count: u16,
    pub p_response_info: *const c_void, // HTTP_RESPONSE_INFO
}

#[repr(C)]
pub struct HttpDataChunk {
    pub data_chunk_type: u32,
    pub p_buffer: *const c_void,
    pub buffer_length: u32,
    pub _padding: [u32; 3],
}

pub const HTTPAPI_VERSION_2: HttpApiVersion = HttpApiVersion {
    http_api_major_version: 2,
    http_api_minor_version: 0,
};

pub const HTTP_VERSION_1_1: HttpVersion = HttpVersion {
    major_version: 1,
    minor_version: 1,
};

pub const HTTP_INITIALIZE_SERVER: u32 = 0x0000_0001;

pub const HTTP_SERVER_BINDING_PROPERTY: u32 = 7;

pub const HTTP_HEADER_REQUEST_MAXIMUM: usize = 41;

pub const HTTP_HEADER_CACHE_CONTROL: usize = 0;
pub const HTTP_HEADER_TRANSFER_ENCODING: usize = 6;
pub const HTTP_HEADER_CONTENT_TYPE: usize = 12;
pub const HTTP_HEADER_RESPONSE_MAXIMUM: usize = 30;

pub const HTTP_REQUEST_FLAG_MORE_ENTITY_BODY_EXISTS: u32 = 0x0000_0001;
pub const HTTP_RECEIVE_REQUEST_ENTITY_BODY_FLAG_FILL_BUFFER: u32 = 0x0000_0001;

pub const HTTP_VERB_GET: u32 = 4;
pub const HTTP_VERB_POST: u32 = 6;

pub const HTTP_SEND_RESPONSE_FLAG_DISCONNECT: u32 = 0x0000_0001;
pub const HTTP_SEND_RESPONSE_FLAG_MORE_DATA: u32 = 0x0000_0002;
pub const HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA: u32 = 0x0000_0004;

pub const HTTP_DATA_CHUNK_FROM_MEMORY: u32 = 0;

pub const NO_ERROR: u32 = 0;
pub const ERROR_HANDLE_EOF: u32 = 38;
pub const ERROR_MORE_DATA: u32 = 234;
