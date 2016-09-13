#version 140

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

in vec3 position;
in vec3 normal;

out vec3 v_normal;
out vec3 v_pos;

void main() {
  mat4 modelview = view * model;
  v_pos = position.xyz;
  v_normal = transpose(inverse(mat3(modelview))) * normal;
  // v_normal = normal;
  gl_Position = perspective * modelview * vec4(position, 1.0);
}
