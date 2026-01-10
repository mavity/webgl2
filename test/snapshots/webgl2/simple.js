// @ts-check

import test from 'node:test';
import assert from 'node:assert/strict';
import { webGL2 /*, getShaderModule, getShaderWat*/ } from '../../../index.js';

test('basic', { skip: true }, async () => {
  const gl = await webGL2();

  // TODO: call some webgl2 APIs to produce some shaders

  const fsActualWat = getShaderWat(gl, gl.FRAGMENT_SHADER, 'xyz');
  assert.equal(fsActualWat, `(module
  (type $t0 (func (param i32 i32 i32) (result i32)))
  (type $t1 (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type $t2 (func (param i32 i32 i32 i32) (result i32)))
  (type $t3 (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type $t4 (func (param i32 i32 i32) (result i32)))
  (type $t5 (func (param i32 i32 i32 i32) (result i32)))
  (type $t6 (func (param i32 i32 i32 i32) (result i32)))
  (type $t7 (func (param i32 i32 i32) (result i32)))
  (type $t8 (func (param i32 i32 i32 i32 i32 i32) (result i32)))
  (type $t9 (func (param i32 i32 i32 i32) (result i32)))
  (type $t10 (func (param i32 i32 i32 i32) (result i32)))
  `);
});