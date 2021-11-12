#version 330

in vec2 v_tex_coords;
uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds
uniform sampler2D iVideo; // Video texture
out vec4 color;


void main() {
    vec2 uv = gl_FragCoord.xy/iResolution.y;

    // Scale to fit width
    uv.x *= iResolution.z;

    // Video source was flipped for some reason
    vec2 tex = v_tex_coords;
    tex.y *= -1.0;

    vec4 pix = texture(iVideo, tex);

    color = vec4(pix.rgb, 1.0);
}
