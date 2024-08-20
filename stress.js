import http from "k6/http"
import { sleep } from "k6"

// const BASE = "http://localhost:3000"
const BASE = "https://graphql.ucsc.menu"

export default function () {
  const q1 = `   
    query Request {
      query {
        locations {
            name
            id
        }
      }
    }`
  http.post(`${BASE}/graphql`, JSON.stringify({ query: q1 }))
  // sleep(0.1)
  const q2 = `   
    query Request {
      query {
        locations {
          menus {
            date
            meals {
              mealType
              sections {
                name
                foodItems {
                    name
                }
              }
            }
          }
        }
      }
    }`
  // sleep(1)
  http.post(`${BASE}/graphql`, JSON.stringify({ query: q2 }))
}
