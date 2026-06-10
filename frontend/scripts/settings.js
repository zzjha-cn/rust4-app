/**
 * Settings page logic
 */

async function loadConfig() {
    try {
        const { invoke } = window.__TAURI_INTERNALS__;
        const config = await invoke('get_config');
        return config;
    } catch (e) {
        console.error('Failed to load config:', e);
        return null;
    }
}

async function saveConfig(config) {
    try {
        const { invoke } = window.__TAURI_INTERNALS__;
        await invoke('update_config', { newConfig: config });
        return true;
    } catch (e) {
        console.error('Failed to save config:', e);
        return false;
    }
}

function populateForm(config) {
    document.getElementById('blink-interval').value = config.blink_interval_sec;
    document.getElementById('rest-interval').value = config.rest_interval_min;
    document.getElementById('ripple-color').value = config.ripple_color;
    document.getElementById('blink-duration').value = config.blink_animation_duration_sec;
    document.getElementById('rest-duration').value = config.rest_animation_duration_sec;
    document.getElementById('enable-work-hours').checked = config.enable_work_hours;
    document.getElementById('work-start').value = config.work_start_hour;
    document.getElementById('work-end').value = config.work_end_hour;

    toggleWorkHours(config.enable_work_hours);
}

function readForm() {
    return {
        blink_interval_sec: parseInt(document.getElementById('blink-interval').value) || 20,
        rest_interval_min: parseInt(document.getElementById('rest-interval').value) || 30,
        blink_animation_duration_sec: parseFloat(document.getElementById('blink-duration').value) || 1.5,
        rest_animation_duration_sec: parseFloat(document.getElementById('rest-duration').value) || 5.0,
        ripple_color: document.getElementById('ripple-color').value,
        work_start_hour: parseInt(document.getElementById('work-start').value) || 9,
        work_end_hour: parseInt(document.getElementById('work-end').value) || 18,
        enable_work_hours: document.getElementById('enable-work-hours').checked,
        enable_sound: false,
        theme: 'light',
    };
}

function toggleWorkHours(enabled) {
    const inputs = document.querySelectorAll('#work-hours-group input');
    for (const input of inputs) {
        input.disabled = !enabled;
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    const config = await loadConfig();
    if (config) {
        populateForm(config);
    }

    document.getElementById('enable-work-hours').addEventListener('change', (e) => {
        toggleWorkHours(e.target.checked);
    });

    document.getElementById('save-btn').addEventListener('click', async () => {
        const newConfig = readForm();
        const success = await saveConfig(newConfig);
        const statusEl = document.getElementById('save-status');
        if (success) {
            statusEl.textContent = '✅ 设置已保存';
            statusEl.style.color = '#34c759';
        } else {
            statusEl.textContent = '❌ 保存失败，请重试';
            statusEl.style.color = '#ff3b30';
        }
        setTimeout(() => { statusEl.textContent = ''; }, 3000);
    });

    document.getElementById('cancel-btn').addEventListener('click', () => {
        window.close();
    });
});