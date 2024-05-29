import { Axios } from 'axios'

const axios = new Axios();

export async function send_message(input: any): Promise<any> {
    await axios.post('/', input)
}