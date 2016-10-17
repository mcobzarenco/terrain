uniform vec3 camera_position;
uniform mat4 perspective;
uniform mat4 view;

layout (location = 0) in vec3 position;
out vec3 tex_coords;

void main()
{
  gl_Position = perspective * view * vec4(camera_position + position, 1.0);
  tex_coords = position;
}
