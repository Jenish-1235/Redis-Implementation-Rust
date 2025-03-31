import { check } from 'k6';
import net from 'k6/net';

export let options = {
    vus: 100,
    duration: '5m',
};

export default function() {
    const client = new net.TCPClient();
    client.connect('localhost', 7171);

    // PUT request
    let putData = JSON.stringify({
        key: `key_${__VU}_${__ITER}`,
        value: 'x'.repeat(256)
    });
    client.write(putData + '\n');
    let putResponse = client.readAll();
    check(putResponse, {
        'PUT success': r => JSON.parse(r).status === 'OK'
    });

    // GET request
    let getData = JSON.stringify({key: `key_${__VU}_${__ITER}`});
    client.write(getData + '\n');
    let getResponse = client.readAll();
    check(getResponse, {
        'GET success': r => JSON.parse(r).status === 'OK'
    });

    client.close();
}