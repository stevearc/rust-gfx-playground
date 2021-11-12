#version 330

in vec2 v_tex_coords;
uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds
out vec4 color;


void main() {
    vec2 uv = gl_FragCoord.xy/iResolution.y;

    // Scale to fit width
    uv.x *= iResolution.z;

    color = vec4(1.0, 0.0, 0.0, 1.0);
}
