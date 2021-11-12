#version 330

in vec2 v_tex_coords;
uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds
uniform sampler2D iTexture; // Texture
out vec4 color;


void main() {
    vec4 pix = texture(iVideo, v_tex_coords);
    color = vec4(pix.rgb, 1.0);
}

