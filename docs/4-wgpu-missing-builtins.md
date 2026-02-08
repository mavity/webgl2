# Missing built‑in variables in wgpu IR 

Based on the WebGL 2.0 spec and the Khronos GLSL/OpenGL refpages here are the canonical built‑in *variables* (with their stage and declarations where available). Findings below.

---

## Canonical built‑in variables (GLSL ES / WebGL 2.0) 🔧
WebGL 2.0 follows GLSL ES 3.00; the OpenGL/Khronos refpages show these common built-ins and their declarations:

- Vertex-stage (inputs / read-only)
  - `gl_VertexID` — `in highp int gl_VertexID;` (vertex index)  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_VertexID.xhtml
  - `gl_InstanceID` — `in highp int gl_InstanceID;` (instance index)  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_InstanceID.xhtml
  - `gl_BaseInstance`, `gl_BaseVertex`, `gl_DrawID` — available in GLES/GL (used by IR variants BaseInstance/BaseVertex/DrawID)

- Vertex-stage (outputs / writable)
  - `gl_Position` — `out highp vec4 gl_Position;`  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_Position.xhtml
  - `gl_PointSize` — `out highp float gl_PointSize;`  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_PointSize.xhtml

- Fragment-stage (inputs / read-only)
  - `gl_FragCoord` — `in highp vec4 gl_FragCoord;`  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_FragCoord.xhtml
  - `gl_FrontFacing` — `in bool gl_FrontFacing;`  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_FrontFacing.xhtml
  - `gl_PointCoord` — `in mediump vec2 gl_PointCoord;`  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_PointCoord.xhtml
  - `gl_FragDepth` — `out highp float gl_FragDepth;` (writeable depth)  
    Reference: https://registry.khronos.org/OpenGL-Refpages/es3.0/html/gl_FragDepth.xhtml

- Index/sample/primitive built‑ins (important for signedness)
  - `gl_PrimitiveID` — declared as `int` in GLSL (fragment / geometry / tessellation contexts)  
    Ref (GL4 refpage showing declaration): https://registry.khronos.org/OpenGL-Refpages/gl4/html/gl_PrimitiveID.xhtml
  - `gl_SampleID` — declared as `in int gl_SampleID;` (fragment)  
    Ref: https://registry.khronos.org/OpenGL-Refpages/gl4/html/gl_SampleID.xhtml
  - `gl_SampleMask` — fragment output array (`int gl_SampleMask[]`) (GL refpage)  
    Ref: https://registry.khronos.org/OpenGL-Refpages/gl4/html/gl_SampleMask.xhtml

- Compute / other built‑ins (examples)
  - `gl_GlobalInvocationID`, `gl_LocalInvocationID`, `gl_WorkGroupID`, etc. (compute stage builtins; available in GLSL ES / GL variants)

