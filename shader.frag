#version 330

#define MAX_STEPS 100
#define MAX_DIST 100.
#define HIT_DIST .01
#define SHADOW_INTENSITY .3

uniform vec3 iResolution; // The viewport resolution (z is pixel aspect ratio, usually 1.0) */
uniform float iTime; // Current time in seconds
out vec4 color;


float SphereDist(vec3 point, float radius) {
    return length(point) - radius;
}

float Fold(float position, float foldloc) {
  return abs(mod(position - foldloc/2., foldloc) - foldloc/2.);
}

float smin(float a, float b, float smoothness) {
  float h = max(smoothness-abs(a-b), 0.0)/smoothness;
  return min(a, b) - h * h * smoothness * (1.0/4.0);
}

float smin(float a, float b) {
  return smin(a, b, 0.1);
}

float GetDist(vec3 point) {
    // Fold space
    point.x = Fold(point.x, 2.);
    point.y = Fold(point.y, 2.);
    point.z = Fold(point.z, 4.);

    float d = SphereDist(point - vec3(.5+.5*cos(iTime), .5+.5*sin(iTime), 2), .4);
    d = smin(d, SphereDist(point - vec3(0.5, 0, 2), .4+.2*sin(.3*iTime)));
    return d;
}

// March rays
float RayMarch(vec3 origin, vec3 ray) {
  float total_distance = 0.;
  for (int i=0; i < MAX_STEPS; i++) {
      vec3 point = origin + ray * total_distance;
      float surface_distance = GetDist(point);
      total_distance += surface_distance;
      if (total_distance > MAX_DIST || abs(surface_distance) < HIT_DIST) break;
  }
  return total_distance;
}

vec3 GetNormal(vec3 point) {
	float distance = GetDist(point);
    vec2 epsilon = vec2(.01, 0);
    // Get three neighboring points and use those to calculate the normal by approximating a flat slope
    vec3 normal = distance - vec3(
        GetDist(point - epsilon.xyy),
        GetDist(point - epsilon.yxy),
        GetDist(point - epsilon.yyx));
    return normalize(normal);
}

float GetLight(vec3 point, vec3 lightPos) {
    vec3 lightVector = normalize(lightPos - point);
    vec3 normal = GetNormal(point);

    float diffuse = clamp(dot(normal, lightVector), 0., 1.);

    if (RayMarch(point + HIT_DIST * 2. * normal, lightVector) < length(lightPos - point)) diffuse *= SHADOW_INTENSITY;
    return diffuse;
}


void main() {
    vec2 uv = (gl_FragCoord.xy-.5*iResolution.xy)/iResolution.y;
    float camera_speed = 0.1;
    vec3 camera_pos = 2.*vec3(cos(camera_speed * iTime), sin(camera_speed * iTime), 0);

    vec3 ray = normalize(vec3(uv.x, uv.y, 1.));
    vec3 light_pos = vec3(cos(iTime), 5, sin(iTime));
    vec3 light_col = vec3(.7, .4, .2);
    vec3 underlight_pos = vec3(-cos(iTime), -10, -sin(iTime));

    float dist = RayMarch(camera_pos, ray);
    vec3 point = camera_pos + ray * dist;
    vec3 light = light_col * GetLight(point, light_pos);
    float underlight = GetLight(point, underlight_pos);

    vec3 col = mix(light, vec3(underlight), .2);

    color = vec4(col,1.0);
}
