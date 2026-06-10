/**
 * PPT Ripple Transition Effect (DOM + CSS Backdrop Filter)
 *
 * 核心特征：
 * - 透明的波纹，没有颜色
 * - 带有放大镜效果（通过 CSS backdrop-filter: blur + brightness 模拟玻璃折射）
 * - 作用于当前窗口，不拦截鼠标事件
 */

class RippleEngine {
    constructor(container) {
        this.container = container;
        this.ripples = [];
    }

    startBlink() {
        this._run({
            count: 2,
            delay: 1200,
            duration: 5200,
        });
    }

    startRest() {
        this._run({
            count: 3,
            delay: 1200,
            duration: 6500,
        });
    }

    _run(params) {
        this.container.innerHTML = '';

        for (let i = 0; i < params.count; i++) {
            setTimeout(() => {
                this._createRipple(params.duration);
            }, i * params.delay);
        }
    }

    _createRipple(duration) {
        const ripple = document.createElement('div');
        ripple.className = 'glass-ripple';

        // 动态设置动画时长
        ripple.style.animationDuration = `${duration}ms`;

        this.container.appendChild(ripple);

        // 动画结束后移除
        setTimeout(() => {
            if (ripple.parentNode) {
                ripple.parentNode.removeChild(ripple);
            }
        }, duration);
    }
}

let engine = null;
let hideTimeout = null;

window.startReminder = function (type) {
    // 每次唤醒时，重置透明度，确保窗口可见
    document.body.style.opacity = '1';

    const container = document.getElementById('ripple-container');
    if (!container) return;

    if (!engine) {
        engine = new RippleEngine(container);
    }

    const messageEl = document.getElementById('message');
    messageEl.classList.remove('show');

    let duration = 3000;

    if (type === 'rest') {
        engine.startRest();
        messageEl.textContent = '该休息啦 ~';
        setTimeout(() => messageEl.classList.add('show'), 300);
        duration = 10400;
    } else {
        engine.startBlink();
        messageEl.textContent = '眨眨眼 ~';
        setTimeout(() => messageEl.classList.add('show'), 200);
        duration = 7600;
    }

    // 每次播放动画时，重新设置自动隐藏的定时器
    if (hideTimeout) clearTimeout(hideTimeout);
    hideTimeout = setTimeout(() => {
        try {
            if (window.__TAURI_INTERNALS__) {
                window.__TAURI_INTERNALS__.invoke('hide_reminder_window');
            }
        } catch (e) { }
        document.body.style.opacity = '0';
    }, duration);
};
