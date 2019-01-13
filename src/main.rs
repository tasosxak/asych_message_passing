use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Condvar};


fn main() {
    let n = 2;
    let m = 1;
    let (tx, rx) = mpsc::channel();
    let mut shout = Vec::with_capacity(n);
    //let mut CS = Arc::new(Mutex(Vec::new()));
    
    for i in 1..=n {
        let (px,zx) = mpsc::channel();
        let thx = mpsc::Sender::clone(&tx);
        shout.push(px);
        //let tcs = Arc::clone(&CS);
        thread::spawn(move || {
             
             extern crate rand;
             use rand::Rng;

             let ts = Arc::new(Mutex::new(1));
             let pid = i;
             let state = Arc::new(Mutex::new(false));
             let ngrants = Arc::new((Mutex::new(0),Condvar::new()));
           //  let in_progress = Arc::new(Mutex::new(false));
             let next_action = Arc::new(Mutex::new(-1));
             let granted_to = Arc::new(Mutex::new(Vec::new()));
             let pending_req = Arc::new(Mutex::new(Vec::new()));
             let preempting_now = Arc::new(Mutex::new(0));
            

             /*
                Exit - Sequence
             */
             let state1 = Arc::clone(&state);
             //let in_progress1 = Arc::clone(&in_progress);
             let thx1 = mpsc::Sender::clone(&thx);
             let (tx1,rx1) = mpsc::channel();
             let (tx1_1,rx1_1) = mpsc::channel();
             let ts1 = Arc::clone(&ts);
             //let tcs1 = Arc::clone(&tcs);
             let _exit_seq_builder = thread::Builder::new().name("ExitSeq".into());
             let _exit_seq = _exit_seq_builder.spawn(move || {

             for _ in rx1  {
                    *(ts1.lock().unwrap()) +=1;
                  
                    let mut state = state1.lock().unwrap();
                    
                    *state = false;
                    println!("Process {} goes out of CS",pid);
                    
                    for i in 1..=n {
                        thx1.send(("Release",0,pid,i)).unwrap();
                    }
                    
                    //*(in_progress1.lock().unwrap()) = false;
                    tx1_1.send("ok");
                }
             }

             );

             /*
                Entry - Sequence
             */

             let state2 = Arc::clone(&state);
             //let in_progress2 = Arc::clone(&in_progress);
             let thx2 = mpsc::Sender::clone(&thx);
             let ngrants2 = Arc::clone(&ngrants);
             let (tx2,rx2) = mpsc::channel();
             let (tx2_1,rx2_1) = mpsc::channel();
             let ts2 = Arc::clone(&ts);
             let _entry_seq_builder = thread::Builder::new().name("EntrySeq".into());
             let _entry_seq = _entry_seq_builder.spawn(move || {

                for _ in rx2  {

                    let &(ref r_ngrants, ref cvar) = &*ngrants2;
                    
                    let mut ngrants = r_ngrants.lock().unwrap();
                    *ngrants = 0;

                    for i in 1..=n {
                        thx2.send(("Request",*(ts2.lock().unwrap()),pid,i)).unwrap();
                    }

                     while *ngrants < n {
                         ngrants = cvar.wait(ngrants).unwrap();
                         println!("Process {} waits" ,pid);
                         
                     }
                     println!("Process {} stops wait" ,pid);
                     let mut state = state2.lock().unwrap();
                     *state = true;
                     println!("Process {} is in CS!",pid);

                    tx2_1.send("ok");
                   // *(in_progress2.lock().unwrap()) = false;
                }
             }

             );

             /*
                Receive - Request
             */
            
             
             let thx3 = mpsc::Sender::clone(&thx);
             let (tx3,rx3) = mpsc::channel();
             let (tx3_1,rx3_1) = mpsc::channel();
             let granted_to_3 = Arc::clone(&granted_to);
             let pending_req_3 = Arc::clone(&pending_req);
             let preempting_now_3 = Arc::clone(&preempting_now);
             let _req_recv_builder = thread::Builder::new().name("Request".into());

             let _req_recv = _req_recv_builder.spawn(move || {
                fn delete_min( v: &mut Vec<(i32,usize)>)->(i32,usize){
                    let mut index = 0;
                    let mut min = (999999,999999);
                    let mut i=0;
                    for t in v.iter() {
                        
                        if t.0 < min.0 || (t.0 == min.0 && t.1 < min.1) {
                            min.0 = t.0;
                            min.1 = t.1;
                            index = i;
                        }
                        i+=1;
                    }
                    v.remove(index);
                    min
                }

                fn max( v: &Vec<(i32,usize)>)->(i32,usize){
                    
                    let mut max = (0,0);

                    for t in v.iter() {
                        
                        if t.0 > max.0 || ((t.0 == max.0) && (t.1 < max.1)) {
                            max.0 = t.0;
                            max.1 = t.1;
                        }
                    }
                    
                    max
                }

                for (timestamp,from) in rx3  {

                    let mut granted_to = granted_to_3.lock().unwrap();
                    let mut pending_req = pending_req_3.lock().unwrap();
                    let mut preempting_now = preempting_now_3.lock().unwrap();
                    pending_req.push((timestamp,from));

                    if granted_to.len() < m {
                        let (tsh,ph) = delete_min(&mut pending_req);
                        granted_to.push((tsh,ph));
                        thx3.send(("Grant",0,pid,ph)).unwrap();
                    }
                    else if *preempting_now == 0 {
                       /* let (tsh,ph) = max(&granted_to);

                        if timestamp < tsh || ((tsh == timestamp) && (from < ph) ) {
                            *preempting_now = ph;
                            thx3.send(("Preempt",0,pid,ph)).unwrap();
                        }
                        */
                    }
                   tx3_1.send("complete");
                }
             }

             );


             /*
                Receive-Grant

             */
            
            let ngrants4 = Arc::clone(&ngrants);
            let (tx4,rx4) = mpsc::channel();
            let (tx4_1,rx4_1) = mpsc::channel();
            let _grant_recv_builder = thread::Builder::new().name("Grant".into());
            let _grant_recv = _grant_recv_builder.spawn(move || {
                for _ in rx4 {

                    let &(ref r_ngrants, ref cvar) = &*ngrants4;
                    let mut ngrants = r_ngrants.lock().unwrap();
                    
                    *ngrants += 1;
                    cvar.notify_one();
                    tx4_1.send("complete");
                    
                }
            });

            /*
                Receive - Release 
            */

            let thx5 = mpsc::Sender::clone(&thx);
            let (tx5,rx5) = mpsc::channel();
            let granted_to_5 = Arc::clone(&granted_to);
            let pending_req_5 = Arc::clone(&pending_req);
            let preempting_now_5 = Arc::clone(&preempting_now);
            let (tx5_1,rx5_1) = mpsc::channel();
            let _rel_recv_builder = thread::Builder::new().name("Release".into());
            let _rel_recv = _rel_recv_builder.spawn(move || {

                 fn delete_min( v: &mut Vec<(i32,usize)>)->(i32,usize){
                    let mut index = 0;
                    let mut min = (999999,999999);
                    let mut i=0;
                    for t in v.iter() {
                        
                        if t.0 < min.0 || (t.0 == min.0 && t.1 < min.1) {
                            min.0 = t.0;
                            min.1 = t.1;
                            index = i;
                        }
                        i+=1;
                    }
                    v.remove(index);
                    min
                }

                for (_timestamp,from) in rx5  {

                    let mut granted_to = granted_to_5.lock().unwrap();
                    let mut pending_req = pending_req_5.lock().unwrap();
                    let mut preempting_now = preempting_now_5.lock().unwrap();
                    
                    if *preempting_now == from {
                        *preempting_now = 0;
                    }
                    
                    let mut i =0;
                    for (_,t) in granted_to.iter() {

                        if *t == from {
                            granted_to.remove(i);
                            break;
                        }
                        i+=1;
                    }

                    if pending_req.len() != 0 {

                        let (tsh,ph) = delete_min(&mut pending_req);
                        granted_to.push((tsh,ph));
                        thx5.send(("Grant",0,pid,ph)).unwrap();
                    }

                    tx5_1.send("complete");
                }

            });


             /*
                   Receive - Preempt
             */

            let thx6 = mpsc::Sender::clone(&thx);
            let (tx6,rx6) = mpsc::channel();
            let state6 = Arc::clone(&state);
            let ngrants6 = Arc::clone(&ngrants);
            let (tx6_1,rx6_1) = mpsc::channel();
            let _preempt_recv_builder = thread::Builder::new().name("Preempt".into());
            let _preempt_recv = _preempt_recv_builder.spawn(move || {

                for (_,from) in rx6 {
                   
                    let state = state6.lock().unwrap();
                    let &(ref r_ngrants, ref cvar) = &*ngrants6;
                    let mut ngrants = r_ngrants.lock().unwrap();

                    if *state == false {
                        println!("{} ngrants val {}",pid,*ngrants);
                        *ngrants -= 1;
                         //cvar.notify_one();
                         thx6.send(("Relinquish",0,pid,from)).unwrap();
                    }
                     
                     
                     tx6_1.send("complete");
                }

             });



             /*
                   Receive - Relinquish
             */

            let thx7 = mpsc::Sender::clone(&thx);
            let (tx7,rx7) = mpsc::channel();
            let granted_to_7 = Arc::clone(&granted_to);
            let pending_req_7 = Arc::clone(&pending_req);
            let preempting_now_7 = Arc::clone(&preempting_now);
            let (tx7_1,rx7_1) = mpsc::channel();
            let _relinq_recv_builder = thread::Builder::new().name("Relinquish".into());
            let _relinq_recv = _relinq_recv_builder.spawn(move || {

                fn delete_min( v: &mut Vec<(i32,usize)>)->(i32,usize){
                    let mut index = 0;
                    let mut min = (999999,999999);
                    let mut i=0;
                    for t in v.iter() {
                        
                        if t.0 < min.0 || (t.0 == min.0 && t.1 < min.1) {
                            min.0 = t.0;
                            min.1 = t.1;
                            index = i;
                        }
                        i+=1;
                    }
                    v.remove(index);
                    min
                }


                for (_,from) in rx7 {
                    let mut granted_to = granted_to_7.lock().unwrap();
                    let mut pending_req = pending_req_7.lock().unwrap();
                    let mut preempting_now = preempting_now_7.lock().unwrap();

                   
                    *preempting_now = 0;

                    let mut i =0;
                    let mut j:(i32,usize) = (0,0);

                    for (_,t) in granted_to.iter() {
                        println!("---->[{}]",from);
                        println!("---->({})",t);
                        if *t == from {
                            let (tsj,pj) = granted_to.remove(i);
                            j.0 = tsj;
                            j.1 = pj;
                            break;
                        }
                        i+=1;
                    }
                    //assert_ne!(j,(0,0));
                    if j.0 != 0 && j.1 != 0 {
                            pending_req.push(j);
                    }
                    
                    let (tsh,ph) = delete_min(&mut pending_req);
                    granted_to.push((tsh,ph));
                    thx7.send(("Grant",0,pid,ph)).unwrap();
                    
                    tx7_1.send("complete");
                }

             });

            // let in_progress_m = Arc::clone(&in_progress);
             let next_action_m = Arc::clone(&next_action);
             let state_m = Arc::clone(&state);

             thread::spawn( move || {
                let mut rng = rand::thread_rng();
                 while true {

                        //println!("{} {}",pid,*(state_m.lock().unwrap()));
                        //println!("{}",*(in_progress.lock().unwrap()) != false);
                        //if *(in_progress_m.lock().unwrap()) == false {
                                
                                
                              //  *(next_action_m.lock().unwrap()) = rng.gen_range(0, 2);

                                //if *(next_action_m.lock().unwrap()) == 1 {

                                    //*(in_progress_m.lock().unwrap()) = true;
                                    
                                    
                                    if *(state_m.lock().unwrap()) == false {
                                    
                                        println!("Process {} wants to insert in CS", pid);
                                        tx2.send("work").unwrap();
                                        for msg in rx2_1.recv() {
                                            break;
                                        }
                                        //thread::sleep(std::time::Duration::from_millis(1000));
                                    }else {
                                        
                                        tx1.send("work").unwrap();
                                        for msg in rx1_1.recv() {
                                            break;
                                        }
                                      //  thread::sleep(std::time::Duration::from_millis(1000));
                                    }      
                          //  }
                   // }

                 }
             });


             //message handler
             loop {
                 
                    
                match zx.try_recv() {
                    Ok((message,timestamp,from,_)) =>  {
                        
                        println!("Process {} received {} from process {}",pid,message,from);
                        

                        if message == "Request" {
                            tx3.send((timestamp,from)).unwrap();
                            for msg in rx3_1.recv() {
                                if msg == "complete" {
                                     println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                                    break;
                                }
                            }
                        }
                        else if message == "Grant" {
                          
                            tx4.send("Grant").unwrap();
                            for msg in rx4_1.recv() {
                                if msg == "complete" {
                                    println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                                    break;
                                }
                            }
                        }
                        else if message == "Release" {
                            tx5.send((timestamp,from)).unwrap();
                            for msg in rx5_1.recv() {
                                if msg == "complete" {
                                    println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                                    break;
                                }
                            }
                        }
                        else if message == "Preempt" {
                            
                            tx6.send((timestamp,from)).unwrap();
                            for msg in rx6_1.recv() {
                                if msg == "complete" {
                                    println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                                    break;
                                }
                            }
                        }
                        else if message == "Relinquish" {
                            tx7.send((timestamp,from)).unwrap();
                            for msg in rx7_1.recv() {
                                if msg == "complete" {
                                    println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                                    break;
                                }
                            }
                        }
                        
                    },
                    Err(_) => continue
                }



             }
            
            
             
        });
        
    }

   for received in rx {
       
        println!("Process {} sends {} to Process {} ", received.2, received.0, received.3);
        shout[received.3-1].send((received.0,received.1,received.2,received.3)).unwrap();
    }
}
