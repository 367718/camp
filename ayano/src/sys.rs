use std::os::raw::*;

unsafe extern "system" {
    
    // -------------------- server -> api --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpinitialize
    pub fn HttpInitialize(
        version: HttpApiVersion,
        flags: c_ulong,
        p_reserved: *mut c_void,
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpterminate
    pub fn HttpTerminate(
        flags: c_ulong,
        p_reserved: *mut c_void,
    ) -> c_ulong;
    
    // -------------------- server -> session --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreateserversession
    pub fn HttpCreateServerSession(
        version: HttpApiVersion,
        p_server_session_id: *mut c_ulonglong, // HTTP_OPAQUE_ID
        reserved: c_ulong,
    ) -> c_ulong;
    
    // https://learn.microsoft.com/es-es/windows/win32/api/http/nf-http-httpcloseserversession
    pub fn HttpCloseServerSession(
        server_session_id: c_ulonglong, // HTTP_OPAQUE_ID
    ) -> c_ulong;
    
    // -------------------- server -> queue --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreaterequestqueue
    pub fn HttpCreateRequestQueue(
        version: HttpApiVersion,
        p_name: *const c_ushort, // PCWSTR
        p_security_attributes: *mut c_void, // SECURITY_ATTRIBUTES
        flags: c_ulong,
        p_req_queue_handle: *mut *mut c_void, // HANDLE
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcloserequestqueue
    pub fn HttpCloseRequestQueue(
        h_req_queue_handle: *mut c_void, // HANDLE
    ) -> c_ulong;
    
    // -------------------- server -> url_group --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcreateurlgroup
    pub fn HttpCreateUrlGroup(
        server_session_id: c_ulonglong, // HTTP_OPAQUE_ID
        p_url_group_id: *mut c_ulonglong, // HTTP_OPAQUE_ID
        reserved: c_ulong,
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpseturlgroupproperty
    pub fn HttpSetUrlGroupProperty(
        url_group_id: c_ulonglong, // HTTP_OPAQUE_ID
        property: c_ulong,
        p_property_information: *const c_void,
        property_information_length: c_ulong,
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpaddurltourlgroup
    pub fn HttpAddUrlToUrlGroup(
        url_group_id: c_ulonglong, // HTTP_OPAQUE_ID
        p_fully_qualified_url: *const c_ushort, // PCWSTR
        url_context: c_ulonglong,
        reserved: c_ulong,
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpcloseurlgroup
    pub fn HttpCloseUrlGroup(
        url_group_id: c_ulonglong, // HTTP_OPAQUE_ID
    ) -> c_ulong;
    
    // -------------------- request --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpreceivehttprequest
    pub fn HttpReceiveHttpRequest(
        request_queue_handle: *mut c_void, // HANDLE
        request_id: c_ulonglong, // HTTP_OPAQUE_ID
        flags: c_ulong,
        p_request_buffer: *mut c_void, // HTTP_REQUEST
        request_buffer_length: c_ulong,
        p_bytes_received: *mut c_ulong,
        p_overlapped: *mut c_void, // OVERLAPPED
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpreceiverequestentitybody
    pub fn HttpReceiveRequestEntityBody(
        request_queue_handle: *mut c_void, // HANDLE
        request_id: c_ulonglong, // HTTP_OPAQUE_ID
        flags: c_ulong,
        p_entity_buffer: *mut c_void,
        entity_buffer_length: c_ulong,
        p_bytes_returned: *mut c_ulong,
        p_overlapped: *mut c_void, // OVERLAPPED
    ) -> c_ulong;
    
    // -------------------- response --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpsendhttpresponse
    pub fn HttpSendHttpResponse(
        request_queue_handle: *mut c_void, // HANDLE
        request_id: c_ulonglong, // HTTP_OPAQUE_ID
        flags: c_ulong,
        p_http_response: *mut HttpResponseV2,
        p_cache_policy: *mut c_void, // HTTP_CACHE_POLICY
        p_bytes_sent: *mut c_ulong,
        p_reserved1: *mut c_void,
        reserved2: c_ulong,
        p_overlapped: *mut c_void, // OVERLAPPED
        p_log_data: *mut c_void, // HTTP_LOG_DATA
    ) -> c_ulong;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/http/nf-http-httpsendresponseentitybody
    pub fn HttpSendResponseEntityBody(
        request_queue_handle: *mut c_void, // HANDLE
        request_id: c_ulonglong, // HTTP_OPAQUE_ID
        flags: c_ulong,
        entity_chunk_count: c_ushort,
        p_entity_chunks: *mut HttpDataChunk,
        p_bytes_sent: *mut c_ulong,
        p_reserved1: *mut c_void,
        reserved2: c_ulong,
        p_overlapped: *mut c_void, // OVERLAPPED
        p_log_data: *mut c_void, // HTTP_LOG_DATA
    ) -> c_ulong;
    
}

#[repr(C)]
pub struct HttpApiVersion {
    pub http_api_major_version: c_ushort,
    pub http_api_minor_version: c_ushort,
}

#[repr(C)]
#[derive(PartialEq)]
pub struct HttpVersion {
    pub major_version: c_ushort,
    pub minor_version: c_ushort,
}

#[repr(C)]
pub struct HttpPropertyFlags {
    pub present: c_ulong,
}

#[repr(C)]
pub struct HttpBindingInfo {
    pub flags: HttpPropertyFlags,
    pub request_queue_handle: *mut c_void,
}

#[repr(C)]
pub struct HttpCookedUrl {
    pub full_url_length: c_ushort,
    pub host_length: c_ushort,
    pub abs_path_length: c_ushort,
    pub query_string_length: c_ushort,
    pub p_full_url: *const u16, // PCWSTR
    pub p_host: *const u16, // PCWSTR
    pub p_abs_path: *const u16, // PCWSTR
    pub p_query_string: *const u16, // PCWSTR
}

#[repr(C)]
pub struct HttpTransportAddress {
    pub p_remote_address: *mut c_void, // PSOCKADDR
    pub p_local_address: *mut c_void, // PSOCKADDR
}

#[repr(C)]
pub struct HttpUnknownHeader {
    pub name_length: c_ushort,
    pub raw_value_length: c_ushort,
    pub p_name: *const c_char, // PCSTR
    pub p_raw_value: *const c_char, // PCSTR
}

#[repr(C)]
pub struct HttpKnownHeader {
    pub raw_value_length: c_ushort,
    pub p_raw_value: *const c_char, // PCSTR
}

#[repr(C)]
pub struct HttpRequestHeaders {
    pub unknown_header_count: c_ushort,
    pub p_unknown_headers: *mut HttpUnknownHeader,
    pub trailer_count: c_ushort,
    pub p_trailers: *mut HttpUnknownHeader,
    pub known_headers: [HttpKnownHeader; HTTP_HEADER_REQUEST_MAXIMUM],
}

#[repr(C)]
pub struct HttpRequestV2 {
    pub flags: c_ulong,
    pub connection_id: c_ulonglong,
    pub request_id: c_ulonglong,
    pub url_context: c_ulonglong,
    pub version: HttpVersion,
    pub verb: c_ulong,
    pub unknown_verb_length: c_ushort,
    pub raw_url_length: c_ushort,
    pub p_unknown_verb: *const c_char, // PCSTR
    pub p_raw_url: *const c_char, // PCSTR
    pub cooked_url: HttpCookedUrl,
    pub address: HttpTransportAddress,
    pub headers: HttpRequestHeaders,
    pub bytes_received: c_ulonglong,
    pub entity_chunk_count: c_ushort,
    pub p_entity_chunks: *mut c_void,
    pub raw_connection_id: c_ulonglong,
    pub p_ssl_info: *mut c_void,
    pub request_info_count: c_ushort,
    pub p_request_info: *mut c_void, // HTTP_REQUEST_INFO
}

#[repr(C)]
pub struct HttpResponseHeaders {
    pub unknown_header_count: c_ushort,
    pub p_unknown_headers: *mut HttpUnknownHeader,
    pub trailer_count: c_ushort,
    pub p_trailers: *mut HttpUnknownHeader,
    pub known_headers: [HttpKnownHeader; HTTP_HEADER_RESPONSE_MAXIMUM],
}

#[repr(C)]
pub struct HttpResponseV2 {
    pub flags: c_ulong,
    pub version: HttpVersion,
    pub status_code: c_ushort,
    pub reason_length: c_ushort,
    pub p_reason: *const c_char, // PCSTR
    pub headers: HttpResponseHeaders,
    pub entity_chunk_count: c_ushort,
    pub p_entity_chunks: *mut HttpDataChunk,
    pub response_info_count: c_ushort,
    pub p_response_info: *mut c_void, // HTTP_RESPONSE_INFO
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HttpDataChunkFromMemory {
    pub p_buffer: *mut c_void,
    pub buffer_length: c_ulong,
}

#[repr(C)]
pub union HttpDataChunkUnion {
    pub from_memory: HttpDataChunkFromMemory,
    pub _padding: [u64; 3],
}

#[repr(C)]
pub struct HttpDataChunk {
    pub data_chunk_type: c_ulong,
    pub union_data: HttpDataChunkUnion,
}

pub const HTTPAPI_VERSION_2: HttpApiVersion = HttpApiVersion {
    http_api_major_version: 2,
    http_api_minor_version: 0,
};

pub const HTTP_VERSION_1_1: HttpVersion = HttpVersion {
    major_version: 1,
    minor_version: 1,
};

pub const HTTP_INITIALIZE_SERVER: c_ulong = 0x0000_0001;

pub const HTTP_SERVER_BINDING_PROPERTY: c_ulong = 7;

pub const HTTP_HEADER_CACHE_CONTROL: usize = 0;
pub const HTTP_HEADER_TRANSFER_ENCODING: usize = 6;
pub const HTTP_HEADER_CONTENT_LENGTH: usize = 11;
pub const HTTP_HEADER_CONTENT_TYPE: usize = 12;
pub const HTTP_HEADER_RESPONSE_MAXIMUM: usize = 30;
pub const HTTP_DATA_CHUNK_FROM_MEMORY: c_ulong = 0;
pub const HTTP_HEADER_REQUEST_MAXIMUM: usize = 41;

pub const HTTP_REQUEST_FLAG_MORE_ENTITY_BODY_EXISTS: c_ulong = 0x0000_0001;
pub const HTTP_RECEIVE_REQUEST_ENTITY_BODY_FLAG_FILL_BUFFER: c_ulong = 0x0000_0001;

pub const HTTP_SEND_RESPONSE_FLAG_DISCONNECT: c_ulong = 0x0000_0001;
pub const HTTP_SEND_RESPONSE_FLAG_MORE_DATA: c_ulong = 0x0000_0002;

pub const NO_ERROR: c_ulong = 0;
pub const ERROR_HANDLE_EOF: c_ulong = 38;
pub const ERROR_MORE_DATA: c_ulong = 234;
