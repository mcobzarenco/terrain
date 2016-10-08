#version 140
#extension GL_OES_standard_derivatives : enable

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

in vec3 position;
in vec3 normal;
in vec3 bary_coord;

out vec3 v_normal;
out vec3 v_pos;
out vec3 v_bary_coord;

void main() {
  mat4 modelview = view * model;
  v_pos = position.xyz;
  v_normal = transpose(inverse(mat3(modelview))) * normal;
  v_bary_coord = bary_coord;
  // v_normal = normal;
  gl_Position = perspective * modelview * vec4(position, 1.0);
}
