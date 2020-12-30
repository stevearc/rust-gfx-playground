#version 140

in vec2 position;
uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}
