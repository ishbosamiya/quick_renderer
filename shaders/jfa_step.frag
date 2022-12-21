#version 330 core

#define JFA_NULL -1.0
#define JFA_NULL_VEC2 vec2(JFA_NULL, JFA_NULL)
#define JFA_NULL_VEC3 vec3(JFA_NULL_VEC2, 0.0)
#define JFA_NULL_VEC4 vec4(JFA_NULL_VEC3, 0.0)
#define FLT_MAX 1.0e+20

uniform sampler2D u_image;
uniform float u_step_size;

in vec2 v_uv;

out vec4 o_frag_color;

void main() {
	float best_dist = FLT_MAX;
	vec4 best_coords = JFA_NULL_VEC4;

	for (int x = -1; x <= 1; x++) {
		for (int y = -1; y <= 1; y++) {
			// must scale the stepping of the uv with the texture size
			vec2 to_check_uv = v_uv + ((vec2(x, y) * u_step_size)) / textureSize(u_image, 0);

			if (to_check_uv.x < 0.0 || to_check_uv.x > 1.0
					|| to_check_uv.y < 0.0 || to_check_uv.y > 1.0) {
				continue;
			}

			vec4 pixel = texture(u_image, to_check_uv);
			vec2 value = pixel.xy;

			if (value != JFA_NULL_VEC2) {
				float dist = length(value - v_uv);
				if (dist < best_dist) {
					best_dist = dist;
					best_coords = pixel;
				}
			}
		}
	}

	o_frag_color = best_coords;
}
