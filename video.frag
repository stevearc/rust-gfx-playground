#version 330

uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds
uniform sampler2D iVideo; // Video texture
out vec4 color;


void main() {
    vec2 uv = gl_FragCoord.xy/iResolution.y;

    // Scale to fit width
    uv.x *= iResolution.z;

    // Video source was flipped for some reason
    uv.y *= -1.0;

    vec4 pix = texture(iVideo, uv.xy);

    color = vec4(pix.rgb, 1.0);
}
