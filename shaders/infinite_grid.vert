// Based on https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

#version 330 core

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

in vec3 in_pos;

// Grid position are in xy clipped space
vec3 gridPlane[6] = vec3[](
													 vec3(1, 1, 0), vec3(-1, -1, 0), vec3(-1, 1, 0),
													 vec3(-1, -1, 0), vec3(1, 1, 0), vec3(1, -1, 0)
													 );

// normal vertice projection
void main() {
	gl_Position = projection * view * vec4(in_pos, 1.0);
}
