//adapted from https://github.com/mattdesl/webgl-lines

#version 140

in vec3 position;
in vec4 color;
in float direction; 
in vec3 next;
in vec3 previous;
out vec4 in_color;
uniform mat4 perspective;
in mat4 view;
in vec4 view_offset;
uniform float aspect;
uniform float thickness;


void main() {
  vec2 aspectVec = vec2(aspect, 1.0);
  mat4 projViewModel = perspective * view;
  vec4 previousProjected = projViewModel * vec4(previous, 1.0);
  vec4 currentProjected = projViewModel * vec4(position, 1.0);
  vec4 nextProjected = projViewModel * vec4(next, 1.0);
  //get 2D screen space with W divide and aspect correction
  vec2 currentScreen = currentProjected.xy * aspectVec;
  vec2 previousScreen = previousProjected.xy * aspectVec;
  vec2 nextScreen = nextProjected.xy * aspectVec;
  float len = thickness;
  float orientation = direction;

  //starting point uses (next - current)
  //vec2 dir = vec2(0.0);
  vec2 dir = normalize(nextScreen - previousScreen);
  
  vec2 normal = vec2(-dir.y, dir.x);
  normal *= len/2.0;
  normal.x /= aspect;

  vec4 offset = vec4(normal * orientation, 0.0, 0.0);
  gl_Position = currentProjected + offset + view_offset;
  gl_PointSize = 1.0;

  in_color = color;
}