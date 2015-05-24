// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::{ Arc, RwLock };
use gfx;
use gfx::tex::Size;
use gfx_device_gl;
use window::{ self, OpenGLWindow };


/// A wrapper around the window that implements `Output`.
pub struct Output<R: gfx::Resources, W: OpenGLWindow> {
    /// Glutin window in the open.
    pub window: Arc<RwLock<W>>,
    frame: gfx::handle::FrameBuffer<R>,
    mask: gfx::Mask,
    supports_gamma_convertion: bool,
    gamma: gfx::Gamma,
}

impl<R: gfx::Resources, W: OpenGLWindow> Output<R, W> {
    /// Try to set the gamma conversion.
    pub fn set_gamma(&mut self, gamma: gfx::Gamma) -> Result<(), ()> {
        if self.supports_gamma_convertion || gamma == gfx::Gamma::Original {
            self.gamma = gamma;
            Ok(())
        }else {
            Err(())
        }
    }
}

impl<R: gfx::Resources, W: OpenGLWindow> gfx::Output<R> for Output<R, W> {
    fn get_handle(&self) -> Option<&gfx::handle::FrameBuffer<R>> {
        Some(&self.frame)
    }

    fn get_size(&self) -> (Size, Size) {
        let window::Size { width, height } = self.window.read().unwrap().size();
        (width as Size, height as Size)
    }

    fn get_mask(&self) -> gfx::Mask {
        self.mask
    }

    fn get_gamma(&self) -> gfx::Gamma {
        self.gamma
    }
}

impl<R: gfx::Resources, W: OpenGLWindow> gfx::Window<R> for Output<R, W> {
    fn swap_buffers(&mut self) {
        self.window.write().unwrap().swap_buffers();
    }
}


/// Result of successful context initialization.
pub type Success<W> = (
    gfx::OwnedStream<
        gfx_device_gl::Device,
        Output<gfx_device_gl::Resources, W>,
    >,
    gfx_device_gl::Device,
    gfx_device_gl::Factory,
);

/// Initialize with a window.
pub fn init_shared<W: OpenGLWindow>(window: Arc<RwLock<W>>) -> Success<W> {
    use gfx::traits::StreamFactory;
    
    let mut window_lock = window.write().unwrap();
    window_lock.make_current();
    let device = gfx_device_gl::Device::new(|s| window_lock.get_proc_address(s));
    let mut factory = device.spawn_factory();
    let out = Output {
        window: window.clone(),
        frame: factory.get_main_frame_buffer(),
        mask: gfx::COLOR | gfx::DEPTH | gfx::STENCIL,
        supports_gamma_convertion: true,
        gamma: gfx::Gamma::Original,
    };
    let stream = factory.create_stream(out);
    (stream, device, factory)
}
