'use strict'

const headers = {
    'User-Agent':
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.3',
}

module.exports = async function (fastify, opts) {
    const JWTModel = require('../../../../models/JWT')
    const UserModel = require('../../../../models/User')
    const jwt = new JWTModel()
    const axios = require('axios').default
    const moment = require('moment')
    const dbpool = require('../../../../db')


    const timestampToMySQLDatetime = (jsTimestamp) => {
        return moment.utc(jsTimestamp).format('YYYY-MM-DD HH:mm:ss');
    }

    // Convierte un MySQL datetime a un timestamp de JavaScript asumiendo que el datetime es en UTC
    const mySQLDatetimeToTimestamp = (mysqlDatetime) => {
        return moment.utc(mysqlDatetime, 'YYYY-MM-DD HH:mm:ss').valueOf();
    }

    // function get AllStock and add X hours to expireOn
    const addTimeToAllStock = async (hours) => {
        let conn
        try {
            conn = await dbpool.getConnection()
            let rows = await conn.query(
                'SELECT * FROM stock_accounts'
            )
            rows = rows.filter(row => row.expireOn ? mySQLDatetimeToTimestamp(row.expireOn) > Date.now() - (hours * 60 * 60 * 1000) : false)
            rows.forEach(async (row) => {
                const expireOn = row.expireOn ? mySQLDatetimeToTimestamp(row.expireOn) : null
                if (expireOn) {
                    const newExpireOn = expireOn + (hours * 60 * 60 * 1000)
                    console.log(`Adding ${hours} hours to ${row.id} from ${new Date(expireOn).toLocaleString()} to ${new Date(newExpireOn).toLocaleString()}`)
                    // await conn.query(
                    //     'UPDATE stock_accounts SET expireOn = ? WHERE id = ?',
                    //     [timestampToMySQLDatetime(newExpireOn), row.id]
                    // )
                }
            })
        } catch (e) {
            console.error('Error on addTimeToAllStock', e)
        } finally {
            if (conn) conn.release()
        }
    }

    // addTimeToAllStock(72)


    const wait = async (ms) => {
        return new Promise(r => {
            setTimeout(r, ms);
        })
    }

    const sendTicketHeartbeat = async (reqPool, machineHash, entitlementId, sv_licenseKeyToken) => {
        try {

            const randomWait = Math.floor(Math.random() * (3000 - 750 + 1) + 750)

            await wait(randomWait)

            await sendEntitlementHeartbeat(reqPool, machineHash, entitlementId)

            const randomTicketWait = Math.floor(Math.random() * (680 - 200 + 1) + 200)

            await wait(randomTicketWait)

            // const ticketHeartbeat =
            //     `gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex=` +
            //     encodeURIComponent(machineHash) +
            //     `&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken=${encodeURIComponent(
            //         sv_licenseKeyToken
            //     )}&token=` + entitlementId

            // const ticketResponse = await axios.post(
            //     'https://lambda.fivem.net/api/ticket/create',
            //     ticketHeartbeat,
            //     {   
            //         headers, httpsAgent: new HttpsProxyAgent({
            //             proxy:
            //                 "http://customer-fivemup:FiveMUP2k23HappySex@dc.pr.oxylabs.io:10000",
            //         }),
            //     }
            // )    

            const ticketHeartbeat = {
                method: 'post',
                url: 'https://lambda.fivem.net/api/ticket/create',
                data: `gameName=gta5&guid=148618792012444134&machineHash=AQAL&machineHashIndex=` +
                    encodeURIComponent(machineHash) +
                    `&server=http%3a%2f%51.91.102.108%3a30120%2f&serverKeyToken=${encodeURIComponent(
                        sv_licenseKeyToken
                    )}&token=` + entitlementId,
                headers: headers,
            };

            const ticketResponse = await reqPool.request(ticketHeartbeat)

            if (ticketResponse.data && ticketResponse.data.ticket) {
                return {
                    success: true,
                    message: 'ticket/created success: ' + ticketResponse.data.ticket.substring(0, 16) + '...'
                }
            } else {
                if (ticketResponse?.data?.error?.includes('banned')) {
                    return {
                        success: false,
                        message: 'ticket/created banned: ' + ticketResponse.data?.error ? ticketResponse.data.error : JSON.stringify(ticketResponse.data)
                    }
                }
                return {
                    success: false,
                    message: 'ticket/created failed: ' + ticketResponse.data?.error ? ticketResponse.data.error : JSON.stringify(ticketResponse.data)
                }
            }
        } catch (e) {
            console.error('Error on lambda/ticket/create', e)
            return {
                success: false,
                message: 'ticket/created failed: ' + e.message
            }
        }
    }

    const sendEntitlementHeartbeat = async (reqPool, machineHash, entitlementId) => {
        try {
            const entitlementHeartbeatRequest = {
                method: 'post',
                url: 'https://lambda.fivem.net/api/validate/entitlement',
                data: 'entitlementId=' +
                    entitlementId +
                    '&f=%7b%7d&gameName=gta5&h2=YyMyxwNpROOEdyxjBu%2bNls1LHzPzx1zTEX7RtDmwD5Eb2MPVgeWNFbNZC3YfGgUnbriTU2jsl7jO0SQ9%2bmDqmU1rLf075r4bxMuKLjcUu2IPy3zVXd2ni2xVJJw8%2bFOoWqaTKIQGggBYEBEBRNOsFNjp6TLqbCwKiqMmc7rl8pLj6SCUm1MpNcBg%2fIE15VmMk4erFf26PdrA4GpAKAP%2fdsM9QaY1GbBnwM4V4xWl8EtLWFPF0XW9xePpm5ZPOjU3OfMAZ2eTF6cNkNsxAGHIMB4VTaKLGWoWmRToEEzbh9wTebY97mYeFdtqF8L%2bnNPVv6y0k4szAwdbInJ2oE73iFj5mZIKLGxqKtNGg9r10nJm2Bk1bTchSWTKlsI%2ffN1vvG6g1fxNDf5%2bJyqGnhktaEMt7L8JTxpgHPuAKtAN795kAM%2fZRgHUUqJzxnH4Ps3jSaMAt5eDpzfdkGvhADFIMMfSEEZ6WqQyvwRw85arnc6IgNYKFlqzGnpsHcWE13elDaRPbgNfMwT7U4Jk31vcfSsadYeqN6Ngad6CeF9zty7GWMklfWcRuaRqtiJvPI3%2fhGymZwPdFHsWvsBEFcbKTWVukjVzaXbuuOH81iY%2fCw7Mbq9A%2f%2fERGFNFW5HXUd9WCZsUooXHJcjVuczxO0BgQLfyEGaaemQSr0RwA3abTe7l5nY4wMC%2fJKkB1AKURTTsJcHhbK0Xrz14b5XOZIZDNlUGQpXweFTMWeualdOAxGUvDnnD0%2fqIZ39zjnPdulZUxCzGt%2fPt1Mt2nsAEJaYq%2fSLBqoahs9UtgGs%2fX9PAqqsnJdsRJ%2bZXKA%2fGfeBr58TCQsDJ8B1CCkqqsmAjItskmOY6w2%2fNGhQw7enImzXwvO4%3d&machineHash=AQAL&machineHashIndex=' +
                    encodeURIComponent(machineHash) +
                    '&rosId=1234',
                headers: headers,
            }

            const response = await reqPool.request(entitlementHeartbeatRequest);
            
            // const response = await axios.post(
            //     'https://lambda.fivem.net/api/validate/entitlement',
            //     entitlementHeartbeat,
            //     { headers }
            // )
            if (response.data) {
                return {
                    success: true,
                    message: 'entitlement/heartbeat success: ' + response?.data
                }
            } else {
                return {
                    success: false,
                    message: 'entitlement/heartbeat failed: ' + response?.data?.error ? response.data.error : JSON.stringify(response.data)
                }
            }
        } catch (e) {
            return {
                success: false,
                message: 'entitlement/heartbeat failed: ' + e.message
            }
        }
    }

    const currentVersion = '0.1.1'

    fastify.post('/heartbeat', async function (request, reply) {
        let conn
        let { bot_id, cfxLicense, version } = request.body

        if (!version) {
            // console.log('Skipping heartbeat for bot ' + bot_id)
            return reply.code(400).send({
                message: `Client outdated, please update to v${currentVersion}, download it from the panel.`,
            })
        } else if (version != currentVersion) {
            // console.log('Skipping heartbeat for bot ' + bot_id)
            return reply.code(400).send({
                message: `You are using outdated client v${version}, please update to v${currentVersion}, download it from the panel.`,
            })
        }


        if (!bot_id || !cfxLicense) {
            return reply.code(400).send({
                message:
                    'Need to provide bot_id, sv_licenseKeyToken and cfxLicense',
            })
        }

        try {
            conn = await request.dbpool.getConnection()

            cfxLicense = cfxLicense.trim()

            const server_rows = await conn.query(
                'SELECT * FROM servers WHERE cfxLicense = ?',
                [cfxLicense]
            )

            if (!server_rows[0]) {
                return reply.code(404).send({
                    message:
                        `No server assigned / found for cfxLicense "${cfxLicense}"`,
                })
            }

            const serverName = server_rows[0].name
            const serverId = server_rows[0].id
            const sv_licenseKeyToken = server_rows[0].sv_licenseKeyToken

            // if (serverId != "bnEjOPF80nME") {
            //     console.log('Skipping heartbeat for bot ' + bot_id + " [" + serverName + "]")
            //     return reply.code(200).send({
            //         success: false,
            //         message: 'tHB - Cfx.re is on maintenace, we are awaiting for finish'
            //     })
            // }

            const bot_data = await conn.query(
                'SELECT * FROM stock_accounts WHERE id = ? AND assignedServer = ?',
                [bot_id, serverId]
            )

            if (!bot_data[0]) {
                return reply.code(404).send({
                    message: 'No bot found with that id',
                })
            }

            let {
                expireOn,
                entitlementId,
                machineHash,
                lastTicketHeartbeat,
                lastEntitlementIdHeartbeat,
            } = bot_data[0]

            // console.log(new Date(lastTicketHeartbeat))

            expireOn = expireOn ? mySQLDatetimeToTimestamp(expireOn) : null
            lastTicketHeartbeat = lastTicketHeartbeat ? mySQLDatetimeToTimestamp(lastTicketHeartbeat) : null
            lastEntitlementIdHeartbeat = lastEntitlementIdHeartbeat ? mySQLDatetimeToTimestamp(lastEntitlementIdHeartbeat) : null

            if (expireOn && expireOn < Date.now()) {
                return reply.code(500).send({
                    message: `Bot ${bot_id} expired on ${new Date(expireOn).toLocaleString()}`,
                })
            }

            // return reply.code(200).send({
            //     success: true,
            //     message: 'Bot is alive',
            // })

            if (conn) conn.release()

            if (lastTicketHeartbeat && lastTicketHeartbeat < Date.now() - 60000 * 5) {
                if (Date.now() - 60000 > lastEntitlementIdHeartbeat) {
                    conn = await request.dbpool.getConnection()
                    await conn.query(
                        'UPDATE stock_accounts SET lastTicketHeartbeat = ? WHERE id = ?',
                        [timestampToMySQLDatetime(Date.now()), bot_id]
                    )
                    if (conn) conn.release()
                    console.log('Sending ticket heartbeat for bot cause timeout since last heartbeat was ' + (lastTicketHeartbeat - Date.now()) / 1000 + ' seconds ago')
                    const ticketHeartbeatResponse = await sendTicketHeartbeat(request.reqPool, machineHash, entitlementId, sv_licenseKeyToken)
                    conn = await request.dbpool.getConnection()
                    if (ticketHeartbeatResponse.success) {
                        console.log(`Ticket heartbeat success for bot ${bot_id}`)
                        await conn.query(
                            'UPDATE stock_accounts SET lastTicketHeartbeat = ? WHERE id = ?',
                            [timestampToMySQLDatetime(Date.now()), bot_id]
                        )
                        return reply.code(200).send({
                            success: true,
                            message: 'tHB success for bot ' + bot_id
                        })
                    } else {
                        console.log(`Ticket heartbeat failed for bot ${bot_id}`)
                        await conn.query(
                            'UPDATE stock_accounts SET lastTicketHeartbeatError = ? WHERE id = ?',
                            [ticketHeartbeatResponse.message, bot_id]
                        )

                        return reply.code(200).send({
                            success: false,
                            message: 'tHB failed for bot ' + bot_id
                        })
                    }
                }
            } else if (lastTicketHeartbeat == null) {
                console.log(`First ticket heartbeat for bot ${bot_id}`)
                conn = await request.dbpool.getConnection()
                await conn.query(
                    'UPDATE stock_accounts SET lastTicketHeartbeat = ? WHERE id = ?',
                    [timestampToMySQLDatetime(Date.now()), bot_id]
                )
                if (conn) conn.release()
                const ticketHeartbeatResponse = await sendTicketHeartbeat(request.reqPool, machineHash, entitlementId, sv_licenseKeyToken)
                conn = await request.dbpool.getConnection()
                if (ticketHeartbeatResponse.success) {
                    console.log(`Ticket heartbeat success for bot ${bot_id}`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastTicketHeartbeat = ? WHERE id = ?',
                        [timestampToMySQLDatetime(Date.now()), bot_id]
                    )

                    return reply.code(200).send({
                        success: true,
                        message: 'tHB success for bot ' + bot_id
                    })
                } else {
                    console.log(`Ticket heartbeat failed for bot ${bot_id}`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastTicketHeartbeatError = ? WHERE id = ?',
                        [ticketHeartbeatResponse.message, bot_id]
                    )

                    return reply.code(200).send({
                        success: false,
                        message: 'tHB failed for bot ' + bot_id
                    })
                }
            }

            if (conn) conn.release()

            if (lastEntitlementIdHeartbeat && Date.now() - 1000 > lastEntitlementIdHeartbeat) {
                const entitlementHeartbeatResponse = await sendEntitlementHeartbeat(request.reqPool, machineHash, entitlementId, sv_licenseKeyToken)
                conn = await request.dbpool.getConnection()
                if (entitlementHeartbeatResponse.success) {
                    console.log(`Entitlement heartbeat success for bot ${bot_id} [${serverName}]`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastEntitlementIdHeartbeat = ? WHERE id = ?',
                        [timestampToMySQLDatetime(Date.now()), bot_id]
                    )

                    return reply.code(200).send({
                        success: true,
                        message: `eHB success for bot ${bot_id}`,
                    })
                } else {
                    console.log(`Entitlement heartbeat failed for bot ${bot_id}`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastEntitlementIdHeartbeatError = ? WHERE id = ?',
                        [entitlementHeartbeatResponse.message, bot_id]
                    )

                    return reply.code(200).send({
                        success: false,
                        message: `eHB failed for bot ${bot_id}`,
                    })
                }
            } else if (lastEntitlementIdHeartbeat == null) {
                console.log(`First entitlement heartbeat for bot ${bot_id}`)
                const entitlementHeartbeatResponse = await sendEntitlementHeartbeat(request.reqPool, machineHash, entitlementId, sv_licenseKeyToken)
                conn = await request.dbpool.getConnection()
                if (entitlementHeartbeatResponse.success) {
                    console.log(`Entitlement heartbeat success for bot ${bot_id} [${serverName}]`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastEntitlementIdHeartbeat = ? WHERE id = ?',
                        [timestampToMySQLDatetime(Date.now()), bot_id]
                    )

                    return reply.code(200).send({
                        success: true,
                        message: `eHB success for bot ${bot_id}`,
                    })
                } else {
                    console.log(`Entitlement heartbeat failed for bot ${bot_id}`)
                    await conn.query(
                        'UPDATE stock_accounts SET lastEntitlementIdHeartbeatError = ? WHERE id = ?',
                        [entitlementHeartbeatResponse.message, bot_id]
                    )

                    return reply.code(200).send({
                        success: false,
                        message: `eHB failed for bot ${bot_id}`,
                    })
                }
            } else {
                console.log('Skipping entitlement heartbeat cause last heartbeat was ' + (Date.now() - lastEntitlementIdHeartbeat) / 1000 + ' seconds ago')
                return reply.code(200).send({
                    success: false,
                    message: 'Skipping HB cause last HB was ' + (Date.now() - lastEntitlementIdHeartbeat) / 1000 + ' seconds ago',
                })
            }

        } catch (err) {
            console.log(`Error in heartbeat for bot ${bot_id} with error ${err}`)
            return reply.code(200).send({
                success: false,
                message: 'Internal server error',
            })
        } finally {
            if (conn) conn.release()
        }
    })
}
