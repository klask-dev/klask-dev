# Shared by the locations of nginx.conf that relay requests to the backend.
# $backend and the resolver come from the file written by entrypoint.sh.
proxy_pass http://$backend;
proxy_http_version 1.1;
proxy_set_header Connection "";
proxy_set_header Host $host;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;

# The backend enforces its own request size limit (MAX_REQUEST_BODY_BYTES).
client_max_body_size 0;

# The server-level add_header directives already set these headers on every
# response of these locations (they declare no add_header, so they inherit
# them). Hide the backend's copies so that each header appears once. The
# backend sends them itself for direct access.
proxy_hide_header X-Frame-Options;
proxy_hide_header X-Content-Type-Options;
proxy_hide_header Referrer-Policy;
proxy_hide_header Content-Security-Policy;

# Longer than the backend request timeout (REQUEST_TIMEOUT_SECS, default 60).
proxy_connect_timeout 5s;
proxy_send_timeout 120s;
proxy_read_timeout 120s;

# JSON error when the backend cannot be reached
error_page 502 503 504 = @backend_unavailable;
