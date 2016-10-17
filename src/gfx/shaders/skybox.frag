in vec3 tex_coords;
out vec4 color;

uniform samplerCube skybox;

void main()
{
  color = texture(skybox, tex_coords) * 0.2;
  // color.rgb = normalize(tex_coords);
}
