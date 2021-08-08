// Based on https://asliceofrendering.com/scene%20helper/2020/01/05/InfiniteGrid/

#version 330 core

in vec3 from_vert_near_point;
in vec3 from_vert_far_point;

out vec4 FragColor;

vec4 grid(vec3 frag_pos_3d, float scale) {
	vec2 coord = frag_pos_3d.xz * scale; // use the scale variable to set the distance between the lines
	vec2 derivative = fwidth(coord);
	vec2 grid = abs(fract(coord - 0.5) - 0.5) / derivative;
	float line = min(grid.x, grid.y);
	float minimumz = min(derivative.y, 1);
	float minimumx = min(derivative.x, 1);
	vec4 color = vec4(0.2, 0.2, 0.2, 1.0 - min(line, 1.0));
	// z axis
	if(frag_pos_3d.x > -0.1 * minimumx && frag_pos_3d.x < 0.1 * minimumx) {
		color.z = 1.0;
	}
	// x axis
	if(frag_pos_3d.z > -0.1 * minimumz && frag_pos_3d.z < 0.1 * minimumz) {
		color.x = 1.0;
	}
	return color;
}

void main() {
	float t = -from_vert_near_point.y / (from_vert_far_point.y - from_vert_near_point.y);
	vec3 frag_pos_3d = from_vert_near_point + t * (from_vert_far_point - from_vert_near_point);
	FragColor = grid(frag_pos_3d, 10) * float(t > 0.0);
}
