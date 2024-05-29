import { send_message } from './common'

function wrap_onebutton_message(inner: any) {
    return {
        "to": "Broadcast",
        "body": {
            "OneButton": inner
        }
    }
}

export async function click() {
    await send_message(wrap_onebutton_message("Click"))
}

export async function clicks(n: number) {
    await send_message(wrap_onebutton_message({
        "Clicks": n
    }))
}

export async function long_press_holding(ms: number) {
    await send_message(wrap_onebutton_message({
        "LongPressedHolding": ms,
    }))
}

export async function long_press_held(ms: number) {
    await send_message(wrap_onebutton_message({
        "LongPressedHeld": ms,
    }))
}