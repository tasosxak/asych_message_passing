/*
* Author: Anastasios Temperekidis
*
* An Asynchronous message-passing distributed algorithm 
* for N-CS (Mutex) problem with channels in Rust
*
*/


extern crate piston_window;

use std::{thread,time};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Condvar};
use piston_window::*;

fn main() {

    let n = 30; //Number of nodes
    let m = 2; //How many nodes can be in their critical section
    let (tx, rx) = mpsc::channel();
    let mut shout = Vec::with_capacity(n);
    let in_cs = Arc::new(Mutex::new(vec![[1.0, 0.0, 0.0, 1.0]; n])); // For GUI


    /* The thread below is for visualizing the state of the system */
    let graph_cs = in_cs.clone();
    thread::spawn (move || {

        let mut window: PistonWindow =
        WindowSettings::new("Hello World!", [512; 2])
            .build().unwrap();

        while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            let mut x = 0.0;
            let mut k = -110.0;
            clear([0.0, 0.0, 0.0, 0.0], g);
            
            for i in 0..n {
             
             if i%10 ==0 {
                 k+= 111.0;
                 x= 0.0;
             }

             rectangle((graph_cs.lock().unwrap())[i], // red
             [110.0*x,k , 90.0, 90.0], // rectangle
             c.transform, g);
             x+= 1.0;

            
            }

        });
    }    
    });

    /* Create N threads. Each thread represents one node */    
    for i in 1..=n {
        let (px,zx) = mpsc::channel();
        let thx = mpsc::Sender::clone(&tx);
        shout.push(px);
       

        //Node
        let update_cs = in_cs.clone();
        thread::spawn(move || {
             
             extern crate rand;
             use rand::Rng;

             let ts = Arc::new(Mutex::new(1)); //timer
             let pid = i; //Node's ID
             let state = Arc::new(Mutex::new(false)); // is in critical section or not?
             let ngrants = Arc::new((Mutex::new(0),Condvar::new())); //to insert the node in its cs needs N-1 grants from all the other nodes and one grant from itself.
             let granted_to = Arc::new(Mutex::new(Vec::new())); // a set of timestamps (ts_j,p_j) for the requests to Pj's entering the cs that Pj's has been granted, but that Pj has not yet released.
             let pending_req = Arc::new(Mutex::new(Vec::new())); // a set of timestamps (ts_j,p_j) for the requests to Pj's entering the cs that are pending.
             let preempting_now = Arc::new(Mutex::new(0)); // a process id such that Pi (this node) preempts a permission for Pj's entering the CS, if the preemption is in progress.


             // Message handlers functions are executed in other thread.
             let state1 = Arc::clone(&state);
             let ngrants1 = Arc::clone(&ngrants);
             let granted_to1 = Arc::clone(&granted_to);
             let pending_req1 = Arc::clone(&pending_req);
             let preempting_now1 = Arc::clone(&preempting_now);
             let thx1 = mpsc::Sender::clone(&thx);

             thread::spawn(move || {

                 /* Functions for general purpose delete min ,find max.. */
                 
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
            

             /*
                Receive-Request
             */
             
             let receive_req = |timestamp:i32,from:usize| {
                    let mut granted_to = granted_to1.lock().unwrap();
                    let mut pending_req = pending_req1.lock().unwrap();
                    let mut preempting_now = preempting_now1.lock().unwrap();
                    pending_req.push((timestamp,from));

                    if granted_to.len() < m {
                        let (tsh,ph) = delete_min(&mut pending_req);
                        granted_to.push((tsh,ph));
                        thx1.send(("Grant",0,pid,ph)).unwrap();
                    }
                    else if *preempting_now == 0 {
                        let (tsh,ph) = max(&granted_to);

                        if timestamp < tsh || ((tsh == timestamp) && (from < ph) ) {
                            *preempting_now = ph;
                            thx1.send(("Preempt",0,pid,ph)).unwrap();
                        }
                        
                    }
                
             };


             /*
                Receive-Grant
             */

            let grant = || {
                    let &(ref r_ngrants, ref cvar) = &*ngrants1;
                    let mut ngrants = r_ngrants.lock().unwrap();
                    
                    *ngrants += 1;
                    cvar.notify_one();
                    
            };
         

            /*
                Receive - Release 
            */

            let release = |from:usize| {

                    let mut granted_to = granted_to1.lock().unwrap();
                    let mut pending_req = pending_req1.lock().unwrap();
                    let mut preempting_now = preempting_now1.lock().unwrap();
                    
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
                        thx1.send(("Grant",0,pid,ph)).unwrap();
                    }

              };


             /*
                   Receive - Preempt
             */

            let preempt = |from:usize| {
                   
                    let state = state1.lock().unwrap();
                    let &(ref r_ngrants, ref cvar) = &*ngrants1;
                    let mut ngrants = r_ngrants.lock().unwrap();

                    if *state == false {
                        println!("{} ngrants val {}",pid,*ngrants);
                        if *ngrants > 0 {
                         *ngrants -= 1;
                         
                        }
                        
                        cvar.notify_one();
                        thx1.send(("Relinquish",0,pid,from)).unwrap();
                         
                    }
                     
                };

            /*
                Receive - Relinquish
            */

            let relinquish = |from:usize| {

                    let mut granted_to = granted_to1.lock().unwrap();
                    let mut pending_req = pending_req1.lock().unwrap();
                    let mut preempting_now = preempting_now1.lock().unwrap();

                    *preempting_now = 0;

                    let mut i =0;
                    let mut j:(i32,usize) = (0,0);

                    for (_,t) in granted_to.iter() {
                        
                        if *t == from {
                            let (tsj,pj) = granted_to.remove(i);
                            j.0 = tsj;
                            j.1 = pj;
                            break;
                        }
                        i+=1;
                    }
                   // assert_ne!(j,(0,0));
                    if j.0 != 0 && j.1 != 0 {
                            pending_req.push(j);
                            
                    }
                   // assert_ne!(pending_req.len(),0);
                    if pending_req.len() > 0 {
                    let (tsh,ph) = delete_min(&mut pending_req);
                    granted_to.push((tsh,ph));
                    thx1.send(("Grant",0,pid,ph)).unwrap();
                    }

                };



            //message handler
             loop {
                 
                match zx.try_recv() {

                    Ok((message,timestamp,from,_)) =>  {
                        
                        println!("Process {} received {} from process {}",pid,message,from);
                        
                        if message == "Request" {
                            receive_req(timestamp,from);
                        }
                        else if message == "Grant" {
                            grant();
                        }
                        else if message == "Release" {
                            release(from);
                        }
                        else if message == "Preempt" {
                            preempt(from);
                        }
                        else if message == "Relinquish" {
                            relinquish(from);
                        }
                        
                        println!("Process {} COMPLETE TASK {} from process {}",pid,message,from);
                        
                    },
                    Err(_) => continue
                }

             }

             });


             /* In this thread of this node,is executed the progress that wants (or not)/repeatly to insert(exit) to(from) CS*/
             let state2 = Arc::clone(&state);
             let ts2 = Arc::clone(&ts);
             let ngrants2 = Arc::clone(&ngrants);

             thread::spawn( move || {
                
                 let mut rng = rand::thread_rng();

                 let exit_seq = || {

                    *(ts2.lock().unwrap()) +=1; //increase timer
                    
                    let mut state = state2.lock().unwrap();
                    
                    *state = false;
                    drop(state);
                    println!("Process {} goes out of CS",pid);
                    
                    for i in 1..=n {
                        
                            thx.send(("Release",0,pid,i)).unwrap();
                        
                    }
                    
                };

                let entry_seq = || {

                    let &(ref r_ngrants, ref cvar) = &*ngrants2;

                    let mut ngrants = r_ngrants.lock().unwrap();
                    *ngrants = 0;

                    for i in 1..=n {
                        
                            thx.send(("Request",*(ts.lock().unwrap()),pid,i)).unwrap();
                    }

                     while *ngrants < n {
                         println!("Process {} waits" ,pid);
                         ngrants = cvar.wait(ngrants).unwrap();
                     }

                     drop(ngrants);
                     println!("Process {} stops wait" ,pid);
                     let mut state = state.lock().unwrap();
                     *state = true;
                   
                };
              
                 loop {
                    let state = state2.lock().unwrap();
                    let choice = rng.gen_range(0, 2000000);
                    if *state == false && choice == 1 {
                        drop(state);
                        println!("Process {} wants to insert in CS", pid);
                        let mut zupdate_cs = update_cs.lock().unwrap();
                        zupdate_cs[pid-1] = [1.0,0.0,0.76,0.53];
                        drop(zupdate_cs);

                        entry_seq();
                        println!("Process {} is in CS!",pid);
                        /* Code for CS */
                        let mut bupdate_cs = update_cs.lock().unwrap();
                        bupdate_cs[pid-1] = [1.0,1.0,0.43,0.33];
                        drop(bupdate_cs);
                        thread::sleep(time::Duration::from_millis(400));
                        
                    }else if *state == true {
                        drop(state);
                        exit_seq();

                        let mut rupdate_cs = update_cs.lock().unwrap();
                        rupdate_cs[pid-1] = [1.0, 0.0, 0.0, 1.0];
                        drop(rupdate_cs);
                        thread::sleep(time::Duration::from_millis(200));
                        
                    }

                   //thread::sleep(std::time::Duration::from_millis(1000));    
                      
                 }
             });

             
        });
        
    }

   //message repeater, to get rid of creating many channels (two channels for each Nodei - Nodej)
   //this assumption does not change the correctness of the algorithm
   for received in rx {
       
        println!("Process {} sends {} to Process {} ", received.2, received.0, received.3);
        shout[received.3-1].send((received.0,received.1,received.2,received.3)).unwrap();
    }
}
