# 3D model view (TMV) Alpha

<p align="center">
  <img src="icon.png" width="400" alt="Первая">
  <img src="v_screen/v2.png" width="400" alt="Вторая">
</p>

```bash
#install program
git clone https://github.com/EnotCoder/game_wgpu.git
cd wgpu

#run program (supported formats: obj, gltf, glb, stl)
cargo run -- ./file.obj ./texture.png
```

## Features
- OBJ (via `tobj`), glTF/glb (via `gltf` crate), STL (binary) — auto-detected by extension
- Orbital camera: left-drag orbit, right-drag pan, scroll zoom
- Blender-style dark UI (F1 toggle): name, path, verts/tris, texture toggle, FPS
- Ground grid on XZ plane with alpha blending
- Model rotation and translation
