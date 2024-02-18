mod timer;
mod utils;

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as MemOrdering};

use crossbeam_queue::SegQueue;
use winit::{
    event::{Event, WindowEvent}, 
    window::{Window, WindowBuilder},
    event_loop::{EventLoop, ControlFlow},
};

/// #### 한국어 </br>
/// 현재 애플리케이션이 실행 중인 경우 `true`값을 가집니다. </br>
/// 
/// #### English (Translation) </br>
/// Has the value `true` if the application is currently running. </br>
/// 
static IS_RUNNING: AtomicBool = AtomicBool::new(true);

/// #### 한국어 </br>
/// 렌더링 루프로 보내는 창 이벤트 대기열 입니다. </br>
/// 
/// #### English (Translation) </br>
/// This is the window event queue that is sent to the rendering loop. </br>
/// 
static EVENT_QUEUE: SegQueue<Event<()>> = SegQueue::new();



fn render_loop(
    window: Arc<Window>, 
    instance: Arc<wgpu::Instance>, 
    surface: Arc<wgpu::Surface>, 
    adapter: Arc<wgpu::Adapter>, 
    device: Arc<wgpu::Device>, 
    queue: Arc<wgpu::Queue>
) {
    // (한국어) 스왑체인 및 프레임 버퍼를 설정합니다.
    // (English Translation) Sets the swapchain and frame buffer. 
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT, 
        format: wgpu::TextureFormat::Bgra8Unorm, 
        width: window.inner_size().width, 
        height: window.inner_size().height, 
        present_mode: wgpu::PresentMode::AutoVsync, 
        desired_maximum_frame_latency: 2, 
        alpha_mode: wgpu::CompositeAlphaMode::Auto, 
        view_formats: vec![], 
    };
    surface.configure(&device, &config);

    // (한국어) 렌더링 루프를 실행합니다.
    // (English Translation) Run the rendering loop.
    log::info!("Run Rendering loop.");
    let mut timer = timer::GameTimer::<50>::new();
    while IS_RUNNING.load(MemOrdering::Acquire) {
        // (한국어) 타이머를 갱신합니다.
        // (English Translation) Updates the timer. 
        timer.tick();

        // (한국어) 창 이벤트를 처리합니다.
        // (English Translation) Handles window events. 
        while let Some(event) = EVENT_QUEUE.pop() {
            /*--- TODO ---*/
        }

        // (한국어) 오브젝트들을 갱신합니다.
        // (English Translation) Updates the objects. 
        /*--- TODO ---*/

        // (한국어) 오브젝트들을 그립니다.
        // (English Translation) Draws the objects.
        window.pre_present_notify();
        /*--- TODO ---*/
    }

    log::info!("Finish Rendering loop.");
}

fn main() {
    env_logger::init();
    log::info!("❖ Application Launching ❖");
    
    // (한국어) 창 시스템을 초기화 합니다.
    // (English Translation) Initializes the window system.
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_visible(true)
            .with_resizable(true)
            .with_title("Lab Project 00")
            .build(&event_loop)
            .unwrap()
    );

    // (한국어) 렌더링 시스템을 초기화 합니다.
    // (English Translation) Initialize the rendering system.
    let window_cloned = window.clone();
    let (instance, surface, adapter, device, queue) = utils::setup_rendering_system(window_cloned);

    // (한국어) 새로운 스레드에서 렌더링 루프를 실행합니다.
    // (English Translation) Runs the rendering loop in a new thread.
    let window_cloned = window.clone();
    let mut join = Some(thread::spawn(move || render_loop(
        window_cloned, 
        instance, 
        surface, 
        adapter, 
        device, 
        queue
    )));

    // (한국어) 윈도우 메시지 루프를 실행합니다.
    // (English Translation) Runs the window message loop.
    log::info!("Run Window message loop.");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run(move |event, elwt| {
        // (한국어) 현재 렌더링 스레드가 실행 중인지 확인합니다.
        // (English Translation) Checks if the current rendering thread is running.
        if join.as_ref().is_some_and(|join| join.is_finished()) {
            // (한국어) 렌더링 스레드를 join 합니다.
            // (English Translation) Join the rendering thread.
            join.take().unwrap().join().unwrap();

            // (한국어) 애플리케이션을 종료합니다.
            // (English Translation) Quit the application.
            elwt.exit();
            return;
        }

        // (한국어) 윈도우 이벤트를 처리합니다.
        // (English Translation) Handles window events. 
        let event_cloned = event.clone();
        match event_cloned {
            Event::NewEvents(_) | Event::AboutToWait => {
                return;
            },
            Event::WindowEvent { window_id, event } 
            if window_id == window.id() => match event {
                WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                    IS_RUNNING.store(false, MemOrdering::Release);
                    elwt.exit();
                    return;
                },
                _ => { /* empty */ }
            },
            _ => { /* empty */ }
        }

        // (한국어) 창 이벤트를 이벤트 대기열에 추가합니다.
        // (English Translation) Add a window event to the event queue. 
        EVENT_QUEUE.push(event);
    }).unwrap();

    log::info!("❖ Application Terminate ❖");
}
